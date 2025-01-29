use crate::interruptor::{InterruptionLevel, Interruptor};
use derive_more::Deref;
use futures::stream::{AbortHandle, AbortRegistration};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("The registration has taken already")]
pub struct RegistrationTaken;

#[derive(Debug, Deref)]
pub struct Controller {
    pub registration: Option<AbortRegistration>,
    #[deref]
    pub stopper: Stopper,
}

impl Default for Controller {
    fn default() -> Self {
        let (handle, registration) = AbortHandle::new_pair();
        let stopper = Stopper {
            active: ActiveFlag::default(),
            handle,
        };
        Self {
            registration: Some(registration),
            stopper,
        }
    }
}

impl Controller {
    pub fn take_registration(&mut self) -> Result<AbortRegistration, RegistrationTaken> {
        self.registration.take().ok_or(RegistrationTaken)
    }
}

#[derive(Debug, Clone, Deref)]
pub struct Stopper {
    #[deref]
    active: ActiveFlag,
    handle: AbortHandle,
}

impl Stopper {
    pub fn stop(&self, force: bool) {
        self.active.flag.store(false, Ordering::Relaxed);
        if force {
            self.handle.abort();
        }
    }
}

impl Interruptor for Stopper {
    fn interrupt(&self) {
        self.stop(false);
    }

    fn interrupt_with_level(&self, level: InterruptionLevel) {
        let force = level > InterruptionLevel::default();
        self.stop(force);
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
