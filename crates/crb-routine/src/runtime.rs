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
        todo!()
    }

    async fn routine(mut self) {
        self.routine.routine(&mut self.context).await;
    }

    fn context(&self) -> &Self::Context {
        &self.context
    }
}
