use crate::agent::Agent;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_core::{mpsc, watch};
use crb_send::{EventSender, Sender};

pub struct AddressJoint<A: Agent> {
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

pub struct Address<A: Agent> {
    msg_tx: mpsc::UnboundedSender<Envelope<A>>,
    status_rx: watch::Receiver<AgentStatus<A>>,
}

impl<A: Agent> Address<A> {
    pub fn send(&self, msg: impl MessageFor<A>) -> Result<()> {
        self.msg_tx
            .send(Box::new(msg))
            .map_err(|_| Error::msg("Can't send the message to the actor"))
    }

    pub async fn join(&mut self) -> Result<Option<A::Output>> {
        let status = self.status_rx.wait_for(AgentStatus::is_done).await?;
        Ok(status.output())
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
    pub fn sender<M>(&self) -> EventSender<M>
    where
        M: MessageFor<A>,
    {
        EventSender::new(self.clone())
    }
}

#[derive(PartialEq, Eq)]
pub enum AgentStatus<T: Agent> {
    Active,
    Interrupted,
    Done(T::Output),
}

impl<T: Agent> AgentStatus<T> {
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Interrupted | Self::Done(_))
    }

    pub fn output(&self) -> Option<T::Output>
    where
        T::Output: Clone,
    {
        match self {
            Self::Active => None,
            Self::Interrupted => None,
            Self::Done(value) => Some(value.clone()),
        }
    }
}

pub type Envelope<A> = Box<dyn MessageFor<A>>;

#[async_trait]
pub trait MessageFor<A: Agent>: Send + 'static {
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut A::Context) -> Result<()>;
}
