//! Generic traits to easily represent different requirements
//! for types of messages.

use anyhow::{Error, Result};

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

/// An `Option` that returns `Error` if is not filled.
pub struct Slot<T> {
    value: Option<T>,
}

impl<T> Slot<T> {
    /// Create a new instance.
    pub fn empty() -> Self {
        Self { value: None }
    }

    /// Set value to the slot.
    pub fn fill(&mut self, value: T) -> Result<()> {
        if self.value.is_some() {
            Err(Error::msg("Slot is already filled"))
        } else {
            self.value = Some(value);
            Ok(())
        }
    }

    /// Get a mutable reference to a value.
    pub fn get_mut(&mut self) -> Result<&mut T> {
        self.value
            .as_mut()
            .ok_or_else(|| Error::msg("Slot is not filled"))
    }
}
