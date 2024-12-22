//! Generic traits to easily represent different requirements
//! for types of messages.

/// A type that implement `'static`
pub trait Staty: 'static {}

impl<T> Staty for T where T: 'static {}

/// A type that implement `Send + 'static`
pub trait Sendy: Send + 'static {}

impl<T> Sendy for T where T: Send + 'static {}

/// A type that implement `Sync + Send + 'static`
pub trait Syncy: Sync + Send + 'static {}

impl<T> Syncy for T where T: Sync + Send + 'static {}

/// A type that implement `Clone + Sync + Send + 'static`
pub trait Clony: Clone + Sync + Send + 'static {}

impl<T> Clony for T where T: Clone + Sync + Send + 'static {}
