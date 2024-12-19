use anyhow::Error;
use derive_more::Deref;
use futures::stream::{AbortHandle, AbortRegistration};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use thiserror::Error;

// TODO: Interruptor has to be the struct, since it's always
// represented by a `Controller`
// So, rename `BasicInterruptor` to `Interruptor`
// and remove the `Interruptor` trait
// and avoid using boxes.

/// The interruptor used internally by a supervisor
/// context or by a standalone routine.
pub trait Interruptor: Send + 'static {
    /// Interrupte a trackable runtime.
    fn stop(&self, force: bool) -> Result<(), Error>;
}

impl Interruptor for AbortHandle {
    fn stop(&self, _force: bool) -> Result<(), Error> {
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

#[derive(Debug, Deref)]
pub struct Controller {
    pub registration: Option<AbortRegistration>,
    #[deref]
    pub interruptor: BasicInterruptor,
}

impl Default for Controller {
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

impl Controller {
    pub fn interruptor(&self) -> Box<dyn Interruptor> {
        Box::new(self.interruptor.clone())
    }

    pub fn take_registration(&mut self) -> Result<AbortRegistration, RegistrationTaken> {
        self.registration.take().ok_or(RegistrationTaken)
    }
}

#[derive(Debug, Clone, Deref)]
pub struct BasicInterruptor {
    #[deref]
    active: ActiveFlag,
    handle: AbortHandle,
}

impl Interruptor for BasicInterruptor {
    fn stop(&self, force: bool) -> Result<(), Error> {
        self.active.flag.store(false, Ordering::Relaxed);
        if force {
            self.handle.abort();
        }
        Ok(())
    }
}
