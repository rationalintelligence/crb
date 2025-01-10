//! Generic traits to easily represent different requirements
//! for types of messages.

use thiserror::Error;

/// A tag that can be sent between threads.
pub trait Tag: Send + 'static {}

impl<T: Send + 'static> Tag for T {}

/// A tag that can be sent between threads.
pub trait SyncTag: Sync + Send + 'static {}

impl<T: Sync + Send + 'static> SyncTag for T {}

/// Errors with a slot. (Missing option's error).
#[derive(Error, Debug)]
pub enum SlotError {
    /// The slot is empty
    #[error("Slot is empty")]
    Empty,
    /// The slot is occupied
    #[error("Slot is occupied")]
    Occupied,
}

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
    pub fn fill(&mut self, value: T) -> Result<(), SlotError> {
        if self.value.is_some() {
            Err(SlotError::Occupied)
        } else {
            self.value = Some(value);
            Ok(())
        }
    }

    /// Clone and take the value.
    pub fn cloned(&self) -> Result<T, SlotError>
    where
        T: Clone,
    {
        self.value.clone().ok_or(SlotError::Empty)
    }

    /// Get a reference to a value.
    pub fn get(&mut self) -> Result<&T, SlotError> {
        self.value.as_ref().ok_or(SlotError::Empty)
    }

    /// Get a mutable reference to a value.
    pub fn get_mut(&mut self) -> Result<&mut T, SlotError> {
        self.value.as_mut().ok_or(SlotError::Empty)
    }
}
