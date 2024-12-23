use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_core::JoinHandle;
use crb_runtime::kit::{Controller, Entrypoint, Failures, Interruptor, Runtime};
use derive_more::{Deref, DerefMut};
use std::marker::PhantomData;
use tokio::task::spawn_blocking;

pub trait SyncTask: Send + 'static {
    fn routine(&mut self) -> Result<()>;
}

pub struct SyncTaskRuntime<T> {
    // The task has to be detachable to move that to a separate thread
    pub task: Option<T>,
    pub controller: Controller,
    pub failures: Failures,
}

impl<T: SyncTask> SyncTaskRuntime<T> {
    pub fn new(task: T) -> Self {
        Self {
            task: Some(task),
            controller: Controller::default(),
            failures: Failures::default(),
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

#[derive(Deref, DerefMut)]
pub struct TypedSyncTask<T> {
    #[deref]
    #[deref_mut]
    task: TypelessSyncTask,
    _run: PhantomData<T>,
}

impl<T: SyncTask> TypedSyncTask<T> {
    pub fn spawn(task: T) -> Self {
        let mut runtime = SyncTaskRuntime::new(task);
        let interruptor = runtime.get_interruptor();
        let handle = crb_core::spawn(runtime.entrypoint());
        let task = TypelessSyncTask {
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

impl<T> From<TypedSyncTask<T>> for TypelessSyncTask {
    fn from(typed: TypedSyncTask<T>) -> Self {
        typed.task
    }
}

pub struct TypelessSyncTask {
    interruptor: Interruptor,
    handle: JoinHandle<()>,
    cancel_on_drop: bool,
}

impl TypelessSyncTask {
    pub fn cancel_on_drop(&mut self, cancel: bool) {
        self.cancel_on_drop = cancel;
    }

    pub fn interrupt(&mut self) {
        self.interruptor.stop(true).ok();
    }
}

impl Drop for TypelessSyncTask {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.handle.abort();
        }
    }
}

impl TypelessSyncTask {
    pub fn spawn<T>(fun: T) -> Self
    where
        T: FnOnce() -> Result<()>,
        T: Send + 'static,
    {
        let task = FnTask { fun: Some(fun) };
        TypedSyncTask::spawn(task).into()
    }
}

struct FnTask<T> {
    fun: Option<T>,
}

impl<T> SyncTask for FnTask<T>
where
    T: FnOnce() -> Result<()>,
    T: Send + 'static,
{
    fn routine(&mut self) -> Result<()> {
        self.fun
            .take()
            .ok_or_else(|| Error::msg("Function has taken already"))?()
    }
}
