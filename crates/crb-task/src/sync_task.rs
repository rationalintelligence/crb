use anyhow::Result;
use async_trait::async_trait;
use crb_runtime::kit::{Controller, Entrypoint, Failures, Interruptor, Runtime};
use tokio::task::spawn_blocking;

pub trait SyncTask: Send + 'static {
    fn routine(&mut self) -> Result<()>;
}

pub struct SyncTaskRuntime<T> {
    pub task: Option<T>,
    pub controller: Controller,
}

impl<T: SyncTask> SyncTaskRuntime<T> {
    pub fn new(task: T) -> Self {
        Self {
            task: Some(task),
            controller: Controller::default(),
        }
    }
}

#[async_trait]
impl<T> Runtime for SyncTaskRuntime<T>
where
    T: SyncTask,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        if let Some(mut task) = self.task.take() {
            let handle = spawn_blocking(move || task.routine());
        }
    }
}
