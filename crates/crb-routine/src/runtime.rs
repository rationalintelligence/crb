use crate::Routine;
use async_trait::async_trait;
use crb_runtime::interruptor::Interruptor;
use crb_runtime::runtime::SupervisedRuntime;

struct RoutineRuntime<T: Routine> {
    routine: T,
    context: T::Context,
}

#[async_trait]
impl<T> SupervisedRuntime for RoutineRuntime<T>
where
    T: Routine,
{
    type Context = T::Context;

    fn get_interruptor(&mut self) -> Box<dyn Interruptor> {
        self.context.controller.interruptor()
    }

    async fn routine(mut self) {
        self.entrypoint().await;
    }

    fn context(&self) -> &Self::Context {
        &self.context
    }
}

impl<T> RoutineRuntime<T>
where
    T: Routine,
{
    async fn entrypoint(mut self) {
        let mut ctx = self.context;
        // log::info!(target: ctx.label(), "Task started");
        let result = self.routine.routine(&mut ctx).await;
        if let Err(err) = self.routine.finalize(result).await {
            // log::error!(target: ctx.label(), "Finalize of the task failed: {}", err);
        }
        // log::info!(target: ctx.label(), "Task finished");
    }
}
