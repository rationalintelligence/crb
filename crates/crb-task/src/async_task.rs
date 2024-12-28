use crate::task::Task;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_core::JoinHandle;
use crb_runtime::kit::{Controller, Entrypoint, Failures, Interruptor, Runtime};
use derive_more::{Deref, DerefMut};
use futures::{stream::Abortable, Future};
use std::marker::PhantomData;

#[async_trait]
pub trait AsyncTask: Send + 'static {
    async fn controlled_routine(&mut self, ctrl: &mut Controller) -> Result<()> {
        let reg = ctrl.take_registration()?;
        let fut = self.routine();
        Abortable::new(fut, reg).await??;
        Ok(())
    }

    async fn routine(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct DoAsync<T> {
    pub task: T,
    pub controller: Controller,
    pub failures: Failures,
}

impl<T: AsyncTask> DoAsync<T> {
    pub fn new(task: T) -> Self {
        Self {
            task,
            controller: Controller::default(),
            failures: Failures::default(),
        }
    }

    /*
    pub fn new_fn<F>(fut: F) -> Self
    where
        F: Future<Output = Result<()>>,
        F: Send + 'static,
    {
        let task = FnAsyncTask { fut: Some(fut) };
        TypedAsyncTask::spawn(task).into()
    }
    */
}

impl<T: AsyncTask> Task<T> for DoAsync<T> {}

#[async_trait]
impl<T> Runtime for DoAsync<T>
where
    T: AsyncTask,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        let res = self.task.controlled_routine(&mut self.controller).await;
        self.failures.put(res);
    }
}

/*
pub struct AsyncFn {
    async_fn: Box<dyn Future<>,
}


impl TypelessAsyncTask {
    pub fn spawn<T>(fut: T) -> Self
    where
        T: Future<Output = Result<()>>,
        T: Send + 'static,
    {
        let task = FnAsyncTask { fut: Some(fut) };
        TypedAsyncTask::spawn(task).into()
    }
}

struct FnAsyncTask<T> {
    fut: Option<T>,
}

#[async_trait]
impl<T> AsyncTask for FnAsyncTask<T>
where
    T: Future<Output = Result<()>>,
    T: Send + 'static,
{
    async fn routine(&mut self) -> Result<()> {
        self.fut
            .take()
            .ok_or_else(|| Error::msg("Future has taken already"))?
            .await
    }
}
*/
