use derive_more::{From, Into};
use futures::stream::AbortHandle;

#[derive(Default, From, Into, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct InterruptionLevel(pub u8);

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
