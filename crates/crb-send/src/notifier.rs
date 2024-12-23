//! A module with notifiers.

use crate::sender::{EventSender, Sender};
use anyhow::Error;
use std::sync::Arc;

/// A notifier that send an associated message to the sender.
pub struct EventNotifier<M> {
    message: M,
    sender: EventSender<M>,
}

impl<M> EventNotifier<M> {
    /// Create a new notifier instance.
    pub fn new<S>(sender: S, message: M) -> Self
    where
        S: Sender<M> + 'static,
    {
        let sender = EventSender::new(sender);
        Self { message, sender }
    }

    /// reates a new notifier instance.
    pub fn new_with_sender(sender: EventSender<M>, message: M) -> Self {
        Self { message, sender }
    }
}

impl<M> EventNotifier<M>
where
    M: Clone,
{
    /// Send a notification.
    pub fn notify(&self) -> Result<(), Error> {
        self.sender.send(self.message.clone())
    }
}

impl<M> EventNotifier<M>
where
    M: Clone + Send + Sync + 'static,
{
    /// Hides the type of a message of the notifier.
    pub fn to_any(self) -> AnyNotifier {
        AnyNotifier {
            notifier: Arc::new(self),
        }
    }
}

// TODO: Add `DropNotifier` - send a notification
// once only on drop
// TODO: `DropNotifier` must be used to hooks

/// An abstract notifier.
pub trait Notifier: Send + Sync {
    /// Send a notification to the recipient.
    fn notify(&self) -> Result<(), Error>;
}

impl<M> Notifier for EventNotifier<M>
where
    M: Clone + Send + Sync + 'static,
{
    fn notify(&self) -> Result<(), Error> {
        self.sender.send(self.message.clone())
    }
}

/// A notifier without a particular type.
pub struct AnyNotifier {
    notifier: Arc<dyn Notifier>,
}

impl AnyNotifier {
    /// Send a notification to a recipient.
    pub fn notify(&self) -> Result<(), Error> {
        self.notifier.notify()
    }
}
