use futures::stream::AbortHandle;

pub trait Interruptor: Send + 'static {
    // TODO: Add levels?
    fn interrupt(&self);

    fn interrupt_with_level(&self, _level: u8) {
        self.interrupt();
    }
}

impl Interruptor for AbortHandle {
    fn interrupt(&self) {
        self.abort();
    }
}
