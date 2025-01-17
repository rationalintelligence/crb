//! A module with notifiers.

use crate::sender::{Recipient, Sender};
use anyhow::{anyhow as err, Result};
use std::sync::{Arc, Mutex};

/// An abstract notifier.
pub trait Notifier: Send + Sync {
    /// Send a notification to the recipient.
    fn notify(&self) -> Result<()>;

    fn typeless(self) -> TypelessNotifier
    where
        Self: Sized + 'static,
    {
        TypelessNotifier {
            notifier: Arc::new(self),
        }
    }
}

/// A notifier without a particular type.
pub struct TypelessNotifier {
    notifier: Arc<dyn Notifier>,
}

impl Notifier for TypelessNotifier {
    fn notify(&self) -> Result<()> {
        self.notifier.notify()
    }
}

pub struct DropNotifier {
    notifier: TypelessNotifier,
}

impl Drop for DropNotifier {
    fn drop(&mut self) {
        self.notifier.notify().ok();
    }
}

pub struct TypedNotifier<M> {
    message: M,
    sender: Recipient<M>,
}

impl<M> TypedNotifier<M> {
    /// Create a new notifier instance.
    pub fn new<S>(sender: S, message: M) -> Self
    where
        S: Sender<M> + 'static,
    {
        let sender = Recipient::new(sender);
        Self { message, sender }
    }

    pub fn once(self) -> OnceNotifier<M> {
        OnceNotifier {
            notifier: Mutex::new(Some(self)),
        }
    }

    pub fn notify_once(self) -> Result<()> {
        self.sender.send(self.message)
    }
}

impl<M> Notifier for TypedNotifier<M>
where
    M: Clone + Send + Sync + 'static,
{
    fn notify(&self) -> Result<()> {
        self.sender.send(self.message.clone())
    }
}

pub struct OnceNotifier<M> {
    notifier: Mutex<Option<TypedNotifier<M>>>,
}

impl<M> OnceNotifier<M>
where
    M: Send + Sync + 'static,
{
    pub fn into_drop_notifier(self) -> DropNotifier {
        DropNotifier {
            notifier: self.typeless(),
        }
    }
}

impl<M> Notifier for OnceNotifier<M>
where
    M: Send + Sync + 'static,
{
    fn notify(&self) -> Result<()> {
        self.notifier
            .lock()
            .map_err(|_| err!("Can't get access to a notifier"))?
            .take()
            .ok_or_else(|| err!("Notification has already sent"))?
            .notify_once()
    }
}
