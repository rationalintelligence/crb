//! Implementation of a unique id.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

/// A unique id.
#[derive(Debug, Default)]
pub struct UniqueId<T = ()>(Arc<T>);

impl<T> Clone for UniqueId<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for UniqueId<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> UniqueId<T> {
    /// Generates a new id.
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl<T> PartialEq for UniqueId<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for UniqueId<T> {}

impl<T> PartialOrd for UniqueId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for UniqueId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let left = Arc::as_ptr(&self.0) as usize;
        let right = Arc::as_ptr(&other.0) as usize;
        left.cmp(&right)
    }
}

impl<T> Hash for UniqueId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Arc::as_ptr(&self.0) as usize;
        ptr.hash(state);
    }
}

impl<T> fmt::Display for UniqueId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = Arc::as_ptr(&self.0) as usize;
        write!(f, "uid:{}", value)
    }
}
