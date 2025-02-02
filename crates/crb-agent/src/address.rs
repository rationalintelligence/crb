use crate::agent::Agent;
use crate::context::Context;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_core::{mpsc, watch};
use crb_runtime::Stopper;
use crb_send::{Recipient, Sender};

pub struct AddressJoint<A: Agent> {
    msg_rx: mpsc::UnboundedReceiver<Envelope<A>>,
    status_tx: watch::Sender<AgentStatus>,
}

impl<A: Agent> AddressJoint<A> {
    pub fn new_pair(stopper: Stopper) -> (Address<A>, AddressJoint<A>) {
        let (msg_tx, msg_rx) = mpsc::unbounded_channel();
        let (status_tx, status_rx) = watch::channel(AgentStatus::Active);
        let address = Address {
            msg_tx,
            status_rx,
            stopper,
        };
        let joint = AddressJoint { msg_rx, status_tx };
        (address, joint)
    }

    pub fn report(&mut self, interrupted: bool) -> Result<()> {
        let status = if interrupted {
            AgentStatus::Interrupted
        } else {
            AgentStatus::Done
        };
        self.status_tx.send(status)?;
        Ok(())
    }

    pub async fn next_envelope(&mut self) -> Option<Envelope<A>> {
        self.msg_rx.recv().await
    }

    pub fn close(&mut self) {
        self.msg_rx.close();
    }
}

pub struct Address<A: Agent> {
    msg_tx: mpsc::UnboundedSender<Envelope<A>>,
    status_rx: watch::Receiver<AgentStatus>,
    stopper: Stopper,
}

impl<A: Agent> Address<A> {
    pub fn send(&self, msg: impl MessageFor<A>) -> Result<()> {
        self.msg_tx
            .send(Box::new(msg))
            .map_err(|_| Error::msg("Can't send the message to the actor"))
    }

    /// Important! `join` must use a reference to allow using it under `DerefMut` trait
    pub async fn join(&mut self) -> Result<AgentStatus> {
        let status = self.status_rx.wait_for(AgentStatus::is_finished).await?;
        Ok(status.clone())
    }

    pub(crate) fn stopper(&self) -> &Stopper {
        &self.stopper
    }
}

impl<A: Agent> Clone for Address<A> {
    fn clone(&self) -> Self {
        Self {
            msg_tx: self.msg_tx.clone(),
            status_rx: self.status_rx.clone(),
            stopper: self.stopper.clone(),
        }
    }
}

impl<A, M> Sender<M> for Address<A>
where
    A: Agent,
    M: MessageFor<A>,
{
    fn send(&self, input: M) -> Result<()> {
        Address::send(self, input)
    }
}

impl<A: Agent> Address<A> {
    pub fn sender<M>(&self) -> Recipient<M>
    where
        M: MessageFor<A>,
    {
        Recipient::new(self.clone())
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum AgentStatus {
    Active,
    Interrupted,
    Done,
}

impl AgentStatus {
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Interrupted | Self::Done)
    }
}

pub type Envelope<A> = Box<dyn MessageFor<A>>;

#[async_trait]
pub trait MessageFor<A: Agent>: Send + 'static {
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut Context<A>) -> Result<()>;
}
