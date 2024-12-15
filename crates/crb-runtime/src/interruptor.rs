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

impl Default for ActiveFlag {
    fn default() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(true)),
        }
    }
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

impl BasicInterruptor {
    pub fn new(handle: AbortHandle) -> Self {
        Self {
            active: ActiveFlag::default(),
            handle,
        }
    }
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
