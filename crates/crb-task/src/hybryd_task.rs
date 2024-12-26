use std::convert::Infallible;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_core::JoinHandle;
use crb_runtime::kit::{Controller, Entrypoint, Failures, Interruptor, Runtime};
use derive_more::{Deref, DerefMut};
use std::marker::PhantomData;
use tokio::task::spawn_blocking;

pub trait HybrydState: Send {}

impl<T> HybrydState for T
where T: Send {}

pub struct Init;

pub struct NextState<T: ?Sized> {
    transition: Box<dyn TransitionFor<T>>,
}

#[async_trait]
trait TransitionFor<T>: Send {
    async fn perform(&mut self, task: T) -> Result<(T, NextState<T>)>;
}

struct SyncRunner<T, S> {
    _task: PhantomData<T>,
    _state: PhantomData<S>,
}

#[async_trait]
impl<T, S> TransitionFor<T> for SyncRunner<T, S>
where
    T: SyncActivity<S>,
    S: HybrydState,
{
    async fn perform(&mut self, mut task: T) -> Result<(T, NextState<T>)> {
        let handle = spawn_blocking(move || {
            let state = task.state();
            (task, state)
        });
        let pair = handle.await?;
        Ok(pair)
    }
}

struct AsyncRunner<T, S> {
    _task: PhantomData<T>,
    _state: PhantomData<S>,
}

#[async_trait]
impl<T, S> TransitionFor<T> for AsyncRunner<T, S>
where
    T: Activity<S>,
    S: HybrydState,
{
    async fn perform(&mut self, mut task: T) -> Result<(T, NextState<T>)> {
        let state = task.state().await;
        Ok((task, state))
    }
}

pub enum GoTo<S> {
    Sync,
    Async,
    Crash(Error),
    Done,
    #[doc(hidden)]
    _Phantom(S, Infallible),
}

#[async_trait]
pub trait HybrydTask: Send + 'static {
    async fn begin(&mut self) -> NextState<Self>;
}

#[async_trait]
pub trait Activity<S>: HybrydTask {
    async fn state(&mut self) -> NextState<Self>;
}

pub trait SyncActivity<S>: HybrydTask {
    fn state(&mut self) -> NextState<Self>;
}

pub struct HybrydTaskRuntime<T> {
    pub task: Option<T>,
    pub controller: Controller,
    pub failures: Failures,
}

impl<T: HybrydTask> HybrydTaskRuntime<T> {
    pub fn new(task: T) -> Self {
        Self {
            task: Some(task),
            controller: Controller::default(),
            failures: Failures::default(),
        }
    }
}

#[async_trait]
impl<T> Runtime for HybrydTaskRuntime<T>
where
    T: HybrydTask,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        if let Some(mut task) = self.task.take() {
            let next_state = task.begin().await;
            let mut res: Result<(T, NextState<T>)> = Ok((task, next_state));
            while let Ok((task, mut next_state)) = res {
                res = next_state.transition.perform(task).await;
            }
        }
    }
}

/*
#[derive(Deref, DerefMut)]
pub struct TypedHybrydTask<T> {
    #[deref]
    #[deref_mut]
    task: TypelessHybrydTask,
    _run: PhantomData<T>,
}

impl<T: HybrydTask> TypedHybrydTask<T> {
    pub fn spawn(task: T) -> Self {
        let mut runtime = HybrydTaskRuntime::new(task);
        let interruptor = runtime.get_interruptor();
        let handle = crb_core::spawn(runtime.entrypoint());
        let task = TypelessHybrydTask {
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

impl<T> From<TypedHybrydTask<T>> for TypelessHybrydTask {
    fn from(typed: TypedHybrydTask<T>) -> Self {
        typed.task
    }
}

pub struct TypelessHybrydTask {
    interruptor: Interruptor,
    handle: JoinHandle<()>,
    cancel_on_drop: bool,
}

impl TypelessHybrydTask {
    pub fn cancel_on_drop(&mut self, cancel: bool) {
        self.cancel_on_drop = cancel;
    }

    pub fn interrupt(&mut self) {
        self.interruptor.stop(true).ok();
    }
}

impl Drop for TypelessHybrydTask {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.handle.abort();
        }
    }
}

impl TypelessHybrydTask {
    pub fn spawn<T>(fun: T) -> Self
    where
        T: FnOnce() -> Result<()>,
        T: Send + 'static,
    {
        let task = FnTask { fun: Some(fun) };
        TypedHybrydTask::spawn(task).into()
    }
}

struct FnTask<T> {
    fun: Option<T>,
}

impl<T> HybrydTask for FnTask<T>
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
*/
