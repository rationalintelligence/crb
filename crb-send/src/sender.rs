//! An implementation of an abstract sender.
//!
//! The crate contains a trait and an implementation of a sender.

use crate::notifier::Notifier;
use anyhow::Error;
use std::fmt;
use std::sync::Arc;

/// An abstract sender.
pub trait EventSender<M>: Send + Sync {
    /// Sends an event (data) to a recipient.
    fn send(&self, input: M) -> Result<(), Error>;
}

/// An empty sender that skips sending.
///
/// Useful when you want to drop messages instead of sending them.
#[derive(Debug)]
pub struct EmptySender;

impl<M> EventSender<M> for EmptySender {
    fn send(&self, _msg: M) -> Result<(), Error> {
        Ok(())
    }
}

/// A wrapper to convert any function to a sender.
pub struct FuncSender<F>(F);

impl<F, IN> EventSender<IN> for FuncSender<F>
where
    F: Fn(IN) -> Result<(), Error>,
    F: Send + Sync,
{
    fn send(&self, input: IN) -> Result<(), Error> {
        (self.0)(input)
    }
}

/// A universal cloneable wrapper for `EventSender`.
pub struct Sender<M> {
    recipient: Arc<dyn EventSender<M>>,
}

impl<M> Clone for Sender<M> {
    fn clone(&self) -> Self {
        Self {
            recipient: self.recipient.clone(),
        }
    }
}

impl<M> fmt::Debug for Sender<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sender")
    }
}

impl<M> Sender<M> {
    /// Wraps a sender with a reference counter.
    pub fn new<E>(sender: E) -> Self
    where
        E: EventSender<M> + 'static,
    {
        Self {
            recipient: Arc::new(sender),
        }
    }

    /// Changes `Sender` to another input type.
    pub fn reform<F, IN>(&self, func: F) -> Sender<IN>
    where
        F: Fn(IN) -> M,
        F: Send + Sync + 'static,
        M: 'static,
    {
        let recipient = self.recipient.clone();
        let func_sender = FuncSender(move |input| {
            let output = func(input);
            recipient.send(output)
        });
        Sender::new(func_sender)
    }

    /// Send an event using inner `EventSender`.
    pub fn send(&self, msg: M) -> Result<(), Error> {
        self.recipient.send(msg)
    }

    /// Creates a sender with pre-created message.
    pub fn to_notifier(self, message: M) -> Notifier<M> {
        Notifier::new_with_sender(self, message)
    }
}
