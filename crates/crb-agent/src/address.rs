use crb_core::{mpsc, watch};
use crate::agent::Agent;
use anyhow::Error;
use async_trait::async_trait;

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

    pub fn report(&mut self, output: A::Output) -> Result<(), Error> {
        let status = AgentStatus::Done(output);
        self.status_tx.send(status)
            .map_err(Error::from)
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
    pub fn send(&self, msg: impl MessageFor<A>) -> Result<(), Error> {
        self.msg_tx
            .send(Box::new(msg))
            .map_err(|_| Error::msg("Can't send the message to the actor"))
    }

    pub async fn join(&mut self) -> Result<A::Output, Error> {
        let status = self.status_rx.wait_for(AgentStatus::is_done).await?;
        status.take().ok_or_else(|| Error::msg("Can't extract the output from the agent"))
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

#[derive(PartialEq, Eq)]
pub enum AgentStatus<T: Agent> {
    Active,
    Done(T::Output),
}

impl<T: Agent> AgentStatus<T> {
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done(_))
    }

    pub fn take(&self) -> Option<T::Output> {
        match self {
            Self::Active => None,
            Self::Done(value) => Some(value.clone()),
        }
    }
}

pub type Envelope<A> = Box<dyn MessageFor<A>>;

#[async_trait]
pub trait MessageFor<A: Agent + ?Sized>: Send + 'static {
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut A::Context) -> Result<(), Error>;
}
