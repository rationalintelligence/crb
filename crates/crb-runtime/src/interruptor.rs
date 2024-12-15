use anyhow::Error;
use futures::stream::AbortHandle;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// The interruptor used internally by a supervisor
/// context or by a standalone routine.
pub trait Interruptor: Send + 'static {
    /// Interrupte a trackable runtime.
    fn interrupt_trackable(&self, force: bool) -> Result<(), Error>;
}

impl Interruptor for AbortHandle {
    fn interrupt_trackable(&self, _force: bool) -> Result<(), Error> {
        self.abort();
        Ok(())
    }
}

pub struct BasicInterruptor {
    active: Arc<AtomicBool>,
    handle: AbortHandle,
}

impl Interruptor for BasicInterruptor {
    fn interrupt_trackable(&self, force: bool) -> Result<(), Error> {
        self.active.store(false, Ordering::Relaxed);
        if force {
            self.handle.abort();
        }
        Ok(())
    }
}
