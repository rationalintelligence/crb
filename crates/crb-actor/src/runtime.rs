use crate::message::{MessageFor, Envelope};
use crate::Actor;
use anyhow::Error;
use crb_core::{mpsc, watch};
use crb_runtime::interruptor::Controller;
use crb_runtime::context::{Context, ManagedContext};


pub struct ActorRuntime<T: Actor> {
    actor: T,
    context: T::Context,
}

impl<T: Actor> ActorRuntime<T> {
    pub async fn entrypoint(mut self) {
        // TODO: Add errors collector
        if let Err(err) = self.actor.initialize(&mut self.context).await {
            log::error!("Initialization of the actor failed: {err}");
        }
        while self.context.controller().is_active() {
            if let Err(err) = self.actor.event(&mut self.context).await {
                log::error!("Event handling for the actor failed: {err}");
            }
        }
        if let Err(err) = self.actor.finalize(&mut self.context).await {
            log::error!("Finalization of the actor failed: {err}");
        }
        if let Err(err) = self.context.session().status_tx.send(ActorStatus::Done) {
            log::error!("Can't change the status of the terminated actor: {err}");
        }
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
    fn controller(&self) -> &Controller {
        &self.controller
    }

    fn shutdown(&mut self) {
        self.msg_rx.close();
    }
}

pub trait ActorContext<T>: ManagedContext {
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
        self.msg_tx.send(Box::new(msg))
            .map_err(|_| Error::msg("Can't send the message to the actor"))
    }

    pub async fn join(&mut self) -> Result<(), Error> {
        self.status_rx
            .wait_for(ActorStatus::is_done)
            .await?;
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
    where Self::Context: From<ActorSession<Self>>;
}

impl<T: Actor + 'static> Standalone for T {
    fn spawn(self) -> Address<Self>
    where Self::Context: From<ActorSession<Self>> {
        let context = ActorSession::new();
        let address = context.address().clone();
        let context = T::Context::from(context);
        let runtime = ActorRuntime { actor: self, context };
        crb_core::spawn(runtime.entrypoint());
        address
    }
}
