//! A crate with senders and notifiers.
//!
//! WARNING! Don't use own `SendError` type! Anti-pattern!
//! Because it hides specific errors of implementations,
//! for example, the actors extension sender returns an error
//! with the priority used to send an event (because there are
//! two priority queues). If we use the `SendError` we have to
//! drop the details!

pub mod notifier;
pub mod sender;

pub use notifier::*;
pub use sender::*;
