//! A context for composable blocks.

use crate::interruptor::Controller;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

/// A commont methods of all contexts and spans for tracing and logging.
///
/// The have provide a reference to a label.
pub trait Context: Send {
    /// An address to interact with the context.
    type Address: Send + Clone;

    /// A label that used for logging all events around the context.
    fn label(&self) -> &Label;

    /// A reference to an address.
    fn address(&self) -> &Self::Address;
}

/// The main features of composable block's context.
///
/// It could be interrupted and contains a method to check a life status of a composable block.
pub trait ManagedContext: Context {
    fn controller(&self) -> &Controller;
    /// Marks a context as interrupted.
    fn shutdown(&mut self);
}

/// `Label` is a `Context` for cases when
/// context is not necessary, but for many
/// runtimes at least `Label` is required
/// for tracing.
impl Context for Label {
    type Address = ();

    fn label(&self) -> &Label {
        &self
    }

    fn address(&self) -> &Self::Address {
        &()
    }
}

/// A unique label of an activity.
///
/// Every task with a context has a unique label.
// TODO: Use tracing/telemetry spans here
#[derive(Debug, Clone)]
pub struct Label {
    // TODO: Add `Span` here
    name: Arc<String>,
}

impl Label {
    /// Creates a new label.
    pub fn new(name: String) -> Self {
        Self {
            name: Arc::new(name),
        }
    }

    /// Creates a new label by stacking the existent and a new value.
    pub fn stack(&self, name: &str) -> Self {
        let name = format!("{}::{}", self.name, name);
        Self {
            name: Arc::new(name),
        }
    }
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.name, &other.name)
    }
}

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}

impl Deref for Label {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &*self.name
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name.fmt(f)
    }
}
