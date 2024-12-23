use anyhow::Result;
use async_trait::async_trait;
use crb_runtime::kit::{Controller, Failures, Interruptor, Runtime};
use futures::stream::Abortable;

#[async_trait]
pub trait Task: Send + 'static {
    async fn controlled_routine(&mut self, ctrl: &mut Controller) -> Result<()> {
        let reg = ctrl.take_registration()?;
        let fut = self.routine();
        Abortable::new(fut, reg).await??;
        Ok(())
    }

    async fn routine(&mut self) -> Result<()>;
}

pub struct TaskRuntime<T> {
    pub task: T,
    pub controller: Controller,
    pub failures: Failures,
}

impl<T: Task> TaskRuntime<T> {
    pub fn new(task: T) -> Self {
        Self {
            task,
            controller: Controller::default(),
            failures: Failures::default(),
        }
    }
}

#[async_trait]
impl<T> Runtime for TaskRuntime<T>
where
    T: Task,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        let res = self.task.controlled_routine(&mut self.controller).await;
        self.failures.put(res);
    }
}
