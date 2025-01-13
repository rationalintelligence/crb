//! An implementation of an abstract sender.
//!
//! The crate contains a trait and an implementation of a sender.

use crate::notifier::TypedNotifier;
use anyhow::Result;
use std::fmt;
use std::sync::Arc;

/// An abstract sender.
pub trait Sender<M>: Send + Sync {
    /// Sends an event (data) to a recipient.
    fn send(&self, input: M) -> Result<()>;

    fn notifier(self, message: M) -> TypedNotifier<M>
    where
        Self: Sized + 'static,
    {
        TypedNotifier::new(self, message)
    }
}

/// An empty sender that skips sending.
///
/// Useful when you want to drop messages instead of sending them.
#[derive(Debug)]
pub struct EmptySender;

impl<M> Sender<M> for EmptySender {
    fn send(&self, _msg: M) -> Result<()> {
        Ok(())
    }
}

/// A wrapper to convert any function to a sender.
pub struct FuncSender<F>(F);

impl<F, IN> Sender<IN> for FuncSender<F>
where
    F: Fn(IN) -> Result<()>,
    F: Send + Sync,
{
    fn send(&self, input: IN) -> Result<()> {
        (self.0)(input)
    }
}

/// A universal cloneable wrapper for `Sender`.
pub struct MessageSender<M> {
    recipient: Arc<dyn Sender<M>>,
}

impl<M> Clone for MessageSender<M> {
    fn clone(&self) -> Self {
        Self {
            recipient: self.recipient.clone(),
        }
    }
}

impl<M> Sender<M> for MessageSender<M> {
    fn send(&self, msg: M) -> Result<()> {
        self.recipient.send(msg)
    }
}

impl<M> fmt::Debug for MessageSender<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MessageSender")
    }
}

impl<M> MessageSender<M> {
    /// Wraps a sender with a reference counter.
    pub fn new<E>(sender: E) -> Self
    where
        E: Sender<M> + 'static,
    {
        Self {
            recipient: Arc::new(sender),
        }
    }

    /// Changes `MessageSender` to another input type.
    pub fn reform<F, IN>(&self, func: F) -> MessageSender<IN>
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
        MessageSender::new(func_sender)
    }
}
