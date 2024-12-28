use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::kit::{Controller, Failures, Interruptor, Runtime, Task};
use tokio::task::spawn_blocking;

pub trait SyncTask: Send + 'static {
    fn routine(&mut self) -> Result<()>;
}

pub struct DoSync<T> {
    // The task has to be detachable to move that to a separate thread
    pub task: Option<T>,
    pub controller: Controller,
    pub failures: Failures,
}

impl<T: SyncTask> DoSync<T> {
    pub fn new(task: T) -> Self {
        Self {
            task: Some(task),
            controller: Controller::default(),
            failures: Failures::default(),
        }
    }
}

impl<T: SyncTask> Task<T> for DoSync<T> {}

#[async_trait]
impl<T> Runtime for DoSync<T>
where
    T: SyncTask,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        if let Some(mut task) = self.task.take() {
            let handle = spawn_blocking(move || task.routine().map(move |()| task));
            let res = handle.await.map_err(Error::from).and_then(|res| res);
            match res {
                Ok(task) => {
                    self.task = Some(task);
                }
                Err(err) => {
                    self.failures.put(Err(err));
                }
            }
        }
    }
}
