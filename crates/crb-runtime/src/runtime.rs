use crate::context::ManagedContext;
use crate::interruptor::Interruptor;
use crate::runnable::Runnable;

pub struct Runtime<R: Runnable> {
    task: R,
    context: R::Context,
}

impl<R: Runnable> Runtime<R> {
    pub fn new(task: R) -> Self
    where
        R::Context: Default,
    {
        let context = R::Context::default();
        Self { task, context }
    }

    pub async fn run(self) {
        // self.task.entrypoint(self.context).await
    }

    // Consider to remove
    fn get_interruptor(&mut self) -> Box<dyn Interruptor>
    where
        R::Context: ManagedContext,
    {
        self.context.controller().interruptor()
    }

    fn context(&self) -> &R::Context {
        &self.context
    }
}
