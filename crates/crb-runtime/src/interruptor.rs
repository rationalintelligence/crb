use anyhow::Error;
use futures::stream::{AbortHandle, AbortRegistration};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use thiserror::Error;

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

#[derive(Error, Debug)]
#[error("The registration has taken already")]
pub struct RegistrationTaken;

#[derive(Debug)]
pub struct BasicController {
    pub registration: Option<AbortRegistration>,
    pub interruptor: BasicInterruptor,
}

impl Default for BasicController {
    fn default() -> Self {
        let (handle, registration) = AbortHandle::new_pair();
        let interruptor = BasicInterruptor {
            active: ActiveFlag::default(),
            handle,
        };
        Self {
            registration: Some(registration),
            interruptor,
        }
    }
}

impl BasicController {
    pub fn interruptor(&self) -> Box<dyn Interruptor> {
        Box::new(self.interruptor.clone())
    }

    pub fn take_registration(&mut self) -> Result<AbortRegistration, RegistrationTaken> {
        self.registration.take().ok_or(RegistrationTaken)
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
