//! Generic traits to easily represent different requirements
//! for types of messages.

pub mod slot;
pub mod unique;

pub use slot::*;
pub use unique::*;

/// A tag that can be sent between threads.
pub trait Tag: Send + 'static {}

impl<T: Send + 'static> Tag for T {}

/// A tag that can be sent between threads.
pub trait SyncTag: Sync + Send + 'static {}

impl<T: Sync + Send + 'static> SyncTag for T {}
