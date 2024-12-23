use crate::runtime::{Task, TaskRuntime};
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_core::JoinHandle;
use crb_runtime::kit::{Entrypoint, Interruptor, Runtime};
use futures::Future;
use std::marker::PhantomData;

pub struct TypedTask<T> {
    task: TypelessTask,
    _run: PhantomData<T>,
}

impl<T: Task> TypedTask<T> {
    pub fn spawn(task: T) -> Self {
        let mut runtime = TaskRuntime::new(task);
        let interruptor = runtime.get_interruptor();
        let handle = crb_core::spawn(runtime.entrypoint());
        let task = TypelessTask {
            interruptor,
            handle,
            cancel_on_drop: false,
        };
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

pub struct TypelessTask {
    interruptor: Interruptor,
    handle: JoinHandle<()>,
    cancel_on_drop: bool,
}

impl TypelessTask {
    pub fn cancel_on_drop(&mut self, cancel: bool) {
        self.cancel_on_drop = cancel;
    }
}

impl Drop for TypelessTask {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.handle.abort();
        }
    }
}

impl TypelessTask {
    pub fn spawn<T>(fut: T) -> Self
    where
        T: Future<Output = Result<()>> + Send + 'static,
    {
        let task = FnTask { fut: Some(fut) };
        TypedTask::spawn(task).into()
    }
}

struct FnTask<T> {
    fut: Option<T>,
}

#[async_trait]
impl<T> Task for FnTask<T>
where
    T: Future<Output = Result<()>> + Send + 'static,
{
    async fn routine(&mut self) -> Result<()> {
        self.fut
            .take()
            .ok_or_else(|| Error::msg("Future has taken already"))?
            .await
    }
}
