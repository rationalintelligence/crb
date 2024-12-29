use crate::actor::Actor;
use crate::message::{Envelope, MessageFor};
use anyhow::Error;
use async_trait::async_trait;
use crb_core::{mpsc, watch};
use crb_runtime::kit::{
    Context, Controller, Failures, InteractiveRuntime, InteractiveTask, Interruptor,
    ManagedContext, Runtime, Task,
};

pub struct DoActor<A: Actor> {
    pub actor: A,
    pub context: A::Context,
    pub failures: Failures,
}

impl<A: Actor> DoActor<A> {
    pub fn new(actor: A) -> Self
    where
        A::Context: Default,
    {
        Self {
            actor,
            context: A::Context::default(),
            failures: Failures::default(),
        }
    }
}

impl<A: Actor> Task<A> for DoActor<A> {}
impl<A: Actor> InteractiveTask<A> for DoActor<A> {}

#[async_trait]
impl<A: Actor> InteractiveRuntime for DoActor<A> {
    type Context = A::Context;

    fn address(&self) -> <Self::Context as Context>::Address {
        self.context.address().clone()
    }
}

#[async_trait]
impl<A: Actor> Runtime for DoActor<A> {
    fn get_interruptor(&mut self) -> Interruptor {
        self.context.controller().interruptor.clone()
    }

    async fn routine(&mut self) {
        let result = self.actor.initialize(&mut self.context).await;
        self.failures.put(result);

        while self.context.session().controller().is_active() {
            let result = self.actor.event(&mut self.context).await;
            self.failures.put(result);
        }

        let result = self.actor.finalize(&mut self.context).await;
        self.failures.put(result);

        let result = self
            .context
            .session()
            .joint
            .status_tx
            .send(ActorStatus::Done)
            .map_err(|_| Error::msg("Can't set actor's status to `Done`"));
        self.failures.put(result);
    }
}

#[derive(PartialEq, Eq)]
pub enum ActorStatus {
    Active,
    Done,
}

impl ActorStatus {
    pub fn is_done(&self) -> bool {
        *self == Self::Done
    }
}

pub struct AddressJoint<A> {
    msg_rx: mpsc::UnboundedReceiver<Envelope<A>>,
    status_tx: watch::Sender<ActorStatus>,
}

impl<A> AddressJoint<A> {
    pub async fn next_envelope(&mut self) -> Option<Envelope<A>> {
        self.msg_rx.recv().await
    }
}

pub struct ActorSession<A> {
    joint: AddressJoint<A>,
    address: Address<A>,
    controller: Controller,
}

impl<A> Default for ActorSession<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> ActorSession<A> {
    pub fn new() -> Self {
        let (msg_tx, msg_rx) = mpsc::unbounded_channel();
        let (status_tx, status_rx) = watch::channel(ActorStatus::Active);
        let controller = Controller::default();
        let address = Address { msg_tx, status_rx };
        let joint = AddressJoint { msg_rx, status_tx };
        Self {
            joint,
            controller,
            address,
        }
    }

    pub fn joint(&mut self) -> &mut AddressJoint<A> {
        &mut self.joint
    }
}

impl<T> Context for ActorSession<T> {
    type Address = Address<T>;

    fn address(&self) -> &Self::Address {
        &self.address
    }
}

impl<T> ManagedContext for ActorSession<T> {
    fn controller(&mut self) -> &mut Controller {
        &mut self.controller
    }

    fn shutdown(&mut self) {
        self.joint.msg_rx.close();
    }
}

pub trait ActorContext<T>: Context<Address = Address<T>> + ManagedContext {
    fn session(&mut self) -> &mut ActorSession<T>;
}

impl<T> ActorContext<T> for ActorSession<T> {
    fn session(&mut self) -> &mut ActorSession<T> {
        self
    }
}

pub struct Address<A: ?Sized> {
    msg_tx: mpsc::UnboundedSender<Envelope<A>>,
    status_rx: watch::Receiver<ActorStatus>,
}

impl<A: Actor> Address<A> {
    pub fn send(&self, msg: impl MessageFor<A>) -> Result<(), Error> {
        self.msg_tx
            .send(Box::new(msg))
            .map_err(|_| Error::msg("Can't send the message to the actor"))
    }

    pub async fn join(&mut self) -> Result<(), Error> {
        self.status_rx.wait_for(ActorStatus::is_done).await?;
        Ok(())
    }
}

impl<A> Clone for Address<A> {
    fn clone(&self) -> Self {
        Self {
            msg_tx: self.msg_tx.clone(),
            status_rx: self.status_rx.clone(),
        }
    }
}
