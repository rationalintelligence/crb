//! A wrapper that makes `Option` compatible with `Result`.

use derive_more::Display;
use thiserror::Error;
use std::any::type_name;

/// A reason of slot interaction fail.
#[derive(Display, Debug)]
pub enum SlotErrorKind {
    /// The slot is empty
    #[display("is empty")]
    Empty,
    /// The slot is occupied
    #[display("is occupied")]
    Occupied,
}

/// Errors with a slot. (Missing option's error).
#[derive(Error, Debug)]
#[error("Slot [{title}] {kind}")]
pub struct SlotError {
    title: &'static str,
    kind: SlotErrorKind,
}

impl SlotError {
    fn empty(title: &'static str) -> Self {
        Self {
            title,
            kind: SlotErrorKind::Empty,
        }
    }

    fn occupied(title: &'static str) -> Self {
        Self {
            title,
            kind: SlotErrorKind::Occupied,
        }
    }
}

/// An `Option` that returns `Error` if is not filled.
pub struct Slot<T> {
    title: &'static str,
    value: Option<T>,
}

impl<T> Slot<T> {
    /// Create a new instance.
    pub fn empty() -> Self {
        Self {
            title: type_name::<T>(),
            value: None,
        }
    }

    /// Create a new instance filled with a value.
    pub fn filled(value: T) -> Self {
        Self {
            title: type_name::<T>(),
            value: Some(value),
        }
    }

    /// Checks if the slot is empty.
    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    /// Checks if the slot is filled.
    pub fn is_filled(&self) -> bool {
        self.value.is_some()
    }

    /// Set value to the slot.
    pub fn fill(&mut self, value: T) -> Result<(), SlotError> {
        if self.value.is_some() {
            Err(SlotError::occupied(&self.title))
        } else {
            self.value = Some(value);
            Ok(())
        }
    }

    /// Take a value out.
    pub fn take(&mut self) -> Result<T, SlotError> {
        self.value
            .take()
            .ok_or_else(|| SlotError::empty(&self.title))
    }

    /// Clone and take the value.
    pub fn cloned(&self) -> Result<T, SlotError>
    where
        T: Clone,
    {
        self.value
            .clone()
            .ok_or_else(|| SlotError::empty(&self.title))
    }

    /// Get a reference to a value.
    pub fn get(&mut self) -> Result<&T, SlotError> {
        self.value
            .as_ref()
            .ok_or_else(|| SlotError::empty(&self.title))
    }

    /// Get a mutable reference to a value.
    pub fn get_mut(&mut self) -> Result<&mut T, SlotError> {
        self.value
            .as_mut()
            .ok_or_else(|| SlotError::empty(&self.title))
    }
}
