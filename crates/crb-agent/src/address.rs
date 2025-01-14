use crate::agent::Agent;
use crate::context::Context;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_core::{mpsc, watch};
use crb_send::{MessageSender, Sender};

pub trait AddressFor<A: Agent> {
    fn address(&self) -> Address<A>;
}

impl<A: Agent> AddressFor<A> for Address<A> {
    fn address(&self) -> Address<A> {
        self.clone()
    }
}

impl<A: Agent> AddressFor<A> for Context<A> {
    fn address(&self) -> Address<A> {
        Context::address(self).clone()
    }
}

pub struct AddressJoint<A: Agent + ?Sized> {
    msg_rx: mpsc::UnboundedReceiver<Envelope<A>>,
    status_tx: watch::Sender<AgentStatus<A>>,
}

impl<A: Agent> AddressJoint<A> {
    pub fn new_pair() -> (Address<A>, AddressJoint<A>) {
        let (msg_tx, msg_rx) = mpsc::unbounded_channel();
        let (status_tx, status_rx) = watch::channel(AgentStatus::Active);
        let address = Address { msg_tx, status_rx };
        let joint = AddressJoint { msg_rx, status_tx };
        (address, joint)
    }

    pub async fn next_envelope(&mut self) -> Option<Envelope<A>> {
        self.msg_rx.recv().await
    }

    pub fn report(&mut self, output: Option<A::Output>) -> Result<()> {
        let status = output
            .map(AgentStatus::Done)
            .unwrap_or(AgentStatus::Interrupted);
        self.status_tx.send(status).map_err(Error::from)
    }

    pub fn close(&mut self) {
        self.msg_rx.close();
    }
}

pub struct Address<A: Agent + ?Sized> {
    msg_tx: mpsc::UnboundedSender<Envelope<A>>,
    status_rx: watch::Receiver<AgentStatus<A>>,
}

impl<A: Agent> Address<A> {
    pub fn send(&self, msg: impl MessageFor<A>) -> Result<()> {
        self.msg_tx
            .send(Box::new(msg))
            .map_err(|_| Error::msg("Can't send the message to the actor"))
    }

    /// Important! `join` must use a reference to allow using it under `DerefMut` trait
    pub async fn join(&mut self) -> Result<AgentOutput<'_, A>> {
        let status = self.status_rx.wait_for(AgentStatus::is_done).await?;
        Ok(AgentOutput { status })
    }
}

pub struct AgentOutput<'a, A: Agent> {
    status: watch::Ref<'a, AgentStatus<A>>,
}

impl<'a, A: Agent> AgentOutput<'a, A> {
    pub fn output(&mut self) -> Option<A::Output>
    where
        A::Output: Clone,
    {
        self.status.output().cloned()
    }
}

impl<A: Agent> Clone for Address<A> {
    fn clone(&self) -> Self {
        Self {
            msg_tx: self.msg_tx.clone(),
            status_rx: self.status_rx.clone(),
        }
    }
}

impl<A, M> Sender<M> for Address<A>
where
    A: Agent,
    M: MessageFor<A>,
{
    fn send(&self, input: M) -> Result<(), Error> {
        Address::send(self, input)
    }
}

impl<A: Agent> Address<A> {
    pub fn sender<M>(&self) -> MessageSender<M>
    where
        M: MessageFor<A>,
    {
        MessageSender::new(self.clone())
    }
}

#[derive(PartialEq, Eq)]
pub enum AgentStatus<T: Agent + ?Sized> {
    Active,
    Interrupted,
    Done(T::Output),
}

impl<T: Agent> AgentStatus<T> {
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Interrupted | Self::Done(_))
    }

    pub fn output(&self) -> Option<&T::Output> {
        match self {
            Self::Active => None,
            Self::Interrupted => None,
            Self::Done(value) => Some(value),
        }
    }
}

pub type Envelope<A> = Box<dyn MessageFor<A>>;

#[async_trait]
pub trait MessageFor<A: Agent>: Send + 'static {
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut Context<A>) -> Result<()>;
}
