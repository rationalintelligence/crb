use crate::message::{MessageFor, Envelope};
use crate::runtime::ActorStatus;
use crate::Actor;
use anyhow::Error;
use crb_core::{mpsc, watch};
use crb_runtime::interruptor::Controller;
use crb_runtime::context::{Context, ManagedContext};

pub struct ActorContext<T> {
    // TODO: wrap to AddressJoint, and hide
    msg_rx: mpsc::UnboundedReceiver<Envelope<T>>,
    pub status_tx: watch::Sender<ActorStatus>,

    controller: Controller,
    address: Address<T>,
}

impl<T> ActorContext<T> {
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

impl<T> Context for ActorContext<T> {
    type Address = Address<T>;

    fn address(&self) -> &Self::Address {
        &self.address
    }
}

impl<T> ManagedContext for ActorContext<T> {
    fn controller(&self) -> &Controller {
        &self.controller
    }

    fn shutdown(&mut self) {
        self.msg_rx.close();
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
