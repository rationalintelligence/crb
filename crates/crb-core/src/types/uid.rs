//! Implementation of a unique id.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// A unique id.
#[derive(Debug, Default, Clone)]
pub struct UniqueId(Arc<()>);

impl UniqueId {
    /// Generates a new id.
    pub fn new() -> Self {
        Self::default()
    }
}

impl PartialEq for UniqueId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for UniqueId {}

impl PartialOrd for UniqueId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UniqueId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = Arc::as_ptr(&self.0) as usize;
        let b = Arc::as_ptr(&other.0) as usize;
        a.cmp(&b)
    }
}

impl Hash for UniqueId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Arc::as_ptr(&self.0) as usize;
        ptr.hash(state);
    }
}
