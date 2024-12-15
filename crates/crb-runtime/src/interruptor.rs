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

#[derive(Debug, Clone)]
pub struct ActiveFlag {
    flag: Arc<AtomicBool>,
}

impl ActiveFlag {
    pub fn is_active(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
pub struct BasicInterruptor {
    active: ActiveFlag,
    handle: AbortHandle,
}

impl Interruptor for BasicInterruptor {
    fn interrupt_trackable(&self, force: bool) -> Result<(), Error> {
        self.active.flag.store(false, Ordering::Relaxed);
        if force {
            self.handle.abort();
        }
        Ok(())
    }
}
