use crate::message::{Envelope, MessageFor};
use crate::Actor;
use anyhow::Error;
use async_trait::async_trait;
use crb_core::{mpsc, watch};
use crb_runtime::{Context, Controller, Failures, Interruptor, ManagedContext, Runtime};

pub struct ActorRuntime<T: Actor> {
    actor: T,
    context: T::Context,
    failures: Failures,
}

impl<T: Actor> ActorRuntime<T> {
    pub fn new(actor: T) -> Self
    where
        T::Context: Default,
    {
        Self {
            actor,
            context: T::Context::default(),
            failures: Failures::default(),
        }
    }
}

#[async_trait]
impl<T: Actor> Runtime for ActorRuntime<T> {
    type Context = T::Context;

    fn get_interruptor(&mut self) -> Box<dyn Interruptor> {
        self.context.controller().interruptor()
    }

    async fn routine(mut self) -> Failures {
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
            .status_tx
            .send(ActorStatus::Done)
            .map_err(|_| Error::msg("Can't set actor's status to `Done`"));
        self.failures.put(result);

        self.failures
    }

    fn context(&self) -> &Self::Context {
        &self.context
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

pub struct ActorSession<T> {
    // TODO: wrap to AddressJoint, and hide
    msg_rx: mpsc::UnboundedReceiver<Envelope<T>>,
    pub status_tx: watch::Sender<ActorStatus>,

    controller: Controller,
    address: Address<T>,
}

impl<T> Default for ActorSession<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ActorSession<T> {
    pub fn new() -> Self {
        let (msg_tx, msg_rx) = mpsc::unbounded_channel();
        let (status_tx, status_rx) = watch::channel(ActorStatus::Active);
        let controller = Controller::default();
        let address = Address { msg_tx, status_rx };
        Self {
            msg_rx,
            status_tx,
            controller,
            address,
        }
    }

    pub async fn next_envelope(&mut self) -> Option<Envelope<T>> {
        self.msg_rx.recv().await
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
        self.msg_rx.close();
    }
}

pub trait ActorContext<T>: ManagedContext<Address = Address<T>> {
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

pub trait Standalone: Actor {
    fn spawn(self) -> Address<Self>
    where
        Self::Context: Default;
}

impl<T: Actor + 'static> Standalone for T {
    fn spawn(self) -> Address<Self>
    where
        Self::Context: Default,
    {
        let mut runtime = ActorRuntime::new(self);
        let address = runtime.context.session().address().clone();
        crb_core::spawn(runtime.entrypoint());
        address
    }
}
