use core::sync::atomic::{AtomicUsize, Ordering};

pub static CRB: Global = Global::new();

pub struct Global {
    long_threshold: AtomicUsize,
}

impl Global {
    const fn new() -> Self {
        Self {
            long_threshold: AtomicUsize::new(usize::MAX),
        }
    }

    pub fn set_long_threshold(&self, ms: usize) {
        self.long_threshold.store(ms, Ordering::Relaxed);
    }

    pub fn get_long_threshold(&self) -> usize {
        self.long_threshold.load(Ordering::Relaxed)
    }
}
