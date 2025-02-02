use futures::stream::AbortHandle;

// TODO: Use bit flags instead!
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct InterruptionLevel(u32);

impl InterruptionLevel {
    pub const EVENT: Self = Self::custom(100);
    pub const FLAG: Self = Self::custom(1_000);
    pub const ABORT: Self = Self::custom(10_000);
    pub const EXIT: Self = Self::custom(100_000);

    pub const fn custom(value: u32) -> Self {
        Self(value)
    }

    pub fn next(&self) -> InterruptionLevel {
        if *self < Self::EVENT {
            Self::EVENT
        } else if *self < Self::FLAG {
            Self::FLAG
        } else if *self < Self::ABORT {
            Self::ABORT
        } else {
            Self::EXIT
        }
    }
}

pub trait Interruptor: Send + 'static {
    // TODO: Add levels?
    fn interrupt(&self);

    fn interrupt_with_level(&self, _level: InterruptionLevel) {
        self.interrupt();
    }
}

impl Interruptor for AbortHandle {
    fn interrupt(&self) {
        self.abort();
    }
}
