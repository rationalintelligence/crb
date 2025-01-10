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
    pub interruptor: Interruptor,
}

impl Default for Controller {
    fn default() -> Self {
        let (handle, registration) = AbortHandle::new_pair();
        let interruptor = Interruptor {
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
    pub fn take_registration(&mut self) -> Result<AbortRegistration, RegistrationTaken> {
        self.registration.take().ok_or(RegistrationTaken)
    }
}

#[derive(Debug, Clone, Deref)]
pub struct Interruptor {
    #[deref]
    active: ActiveFlag,
    handle: AbortHandle,
}

impl Interruptor {
    pub fn stop(&self, force: bool) {
        self.active.flag.store(false, Ordering::Relaxed);
        if force {
            self.handle.abort();
        }
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
