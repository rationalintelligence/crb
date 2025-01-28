use futures::stream::AbortHandle;

pub trait Interruptor: Send + 'static {
    // TODO: Add levels?
    fn interrupt(&self);
}

impl Interruptor for AbortHandle {
    fn interrupt(&self) {
        self.abort();
    }
}
