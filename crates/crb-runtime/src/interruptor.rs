pub trait Interruptor: Send + 'static {
    // TODO: Add levels?
    fn interrupt(&self);
}
