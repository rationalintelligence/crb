use crate::task::TypelessTask;
use anyhow::Error;
use async_trait::async_trait;
use std::marker::PhantomData;

pub struct TypedTask<T> {
    task: TypelessTask,
    _run: PhantomData<T>,
}

impl<T: Task> TypedTask<T> {
    pub fn spawn(task: T) -> Self {
        let task = TypelessTask::spawn(task.entrypoint());
        Self {
            task,
            _run: PhantomData,
        }
    }
}

impl<T> From<TypedTask<T>> for TypelessTask {
    fn from(typed: TypedTask<T>) -> Self {
        typed.task
    }
}

#[async_trait]
pub trait Task: Send + 'static {
    async fn entrypoint(mut self) -> Result<(), Error>;
}
