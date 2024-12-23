//! A crate with senders and notifiers.
//!
//! WARNING! Don't use own `SendError` type! Anti-pattern!
//! Because it hides specific errors of implementations,
//! for example, the actors extension sender returns an error
//! with the priority used to send an event (because there are
//! two priority queues). If we use the `SendError` we have to
//! drop the details!

pub mod message;
pub mod notifier;
pub mod sender;

pub mod kit {
    pub use crate::message::*;
    pub use crate::notifier::*;
    pub use crate::sender::*;
}
