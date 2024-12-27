use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_core::JoinHandle;
use crb_runtime::kit::{Controller, Entrypoint, Failures, Interruptor, Runtime};
use std::marker::PhantomData;
use tokio::task::spawn_blocking;
use derive_more::{Deref, DerefMut};

pub trait HybrydState: Send + 'static {}

impl<T> HybrydState for T
where T: Send + 'static {}

pub struct NextState<T: ?Sized> {
    transition: Box<dyn TransitionFor<T>>,
}

impl<T> NextState<T>
where
    T: HybrydTask,
{
    pub fn do_sync<S>(_state: S) -> Self
    where
        T: SyncActivity<S>,
        S: HybrydState,
    {
        let runner = SyncRunner {
            _task: PhantomData,
            _state: PhantomData,
        };
        Self {
            transition: Box::new(runner),
        }
    }

    pub fn do_async<S>(_state: S) -> Self
    where
        T: Activity<S>,
        S: HybrydState,
    {
        let runner = AsyncRunner {
            _task: PhantomData,
            _state: PhantomData,
        };
        Self {
            transition: Box::new(runner),
        }
    }

    pub fn done() -> Self {
        Self::interrupt(None)
    }

    pub fn fail(err: Error) -> Self {
        Self::interrupt(Some(err))
    }
}

impl<T> NextState<T>
where
    T: HybrydTask,
{
    fn interrupt(error: Option<Error>) -> Self {
        Self {
            transition: Box::new(Interrupt { error }),
        }
    }
}

pub struct Interrupt {
    error: Option<Error>,
}

#[async_trait]
impl<T> TransitionFor<T> for Interrupt
where
    T: HybrydTask,
{
    async fn perform(&mut self, _task: T) -> Transition<T> {
        match self.error.take() {
            None => Transition::Interrupted,
            Some(err) => Transition::Crashed(err),
        }
    }

    async fn fallback(&mut self, task: T, err: Error) -> (T, NextState<T>) {
        let error = self.error.take().unwrap_or(err);
        (task, NextState::interrupt(Some(error)))
    }
}

enum Transition<T> {
    Next(T, Result<NextState<T>>),
    Crashed(Error),
    Interrupted,
}

#[async_trait]
trait TransitionFor<T>: Send {
    async fn perform(&mut self, task: T) -> Transition<T>;
    async fn fallback(&mut self, task: T, err: Error) -> (T, NextState<T>);
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
    async fn perform(&mut self, mut task: T) -> Transition<T> {
        let handle = spawn_blocking(move || {
            let state = task.perform();
            Transition::Next(task, state)
        });
        match handle.await {
            Ok(transition) => transition,
            Err(err) => Transition::Crashed(err.into()),
        }
    }

    async fn fallback(&mut self, mut task: T, err: Error) -> (T, NextState<T>) {
        let next_state = task.fallback(err);
        (task, next_state)
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
    async fn perform(&mut self, mut task: T) -> Transition<T> {
        let state = task.perform().await;
        Transition::Next(task, state)
    }

    async fn fallback(&mut self, mut task: T, err: Error) -> (T, NextState<T>) {
        let next_state = task.fallback(err).await;
        (task, next_state)
    }
}

#[async_trait]
pub trait HybrydTask: Sized + Send + 'static {
    async fn begin(&mut self) -> NextState<Self>;
}

#[async_trait]
pub trait Activity<S>: HybrydTask {
    async fn perform(&mut self) -> Result<NextState<Self>>;

    async fn fallback(&mut self, err: Error) -> NextState<Self> {
        NextState::fail(err)
    }
}

pub trait SyncActivity<S>: HybrydTask {
    fn perform(&mut self) -> Result<NextState<Self>>;

    fn fallback(&mut self, err: Error) -> NextState<Self> {
        NextState::fail(err)
    }
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
            let mut pair = (task, next_state);
            loop {
                let (task, mut next_state) = pair;
                let res = next_state.transition.perform(task).await;
                match res {
                    Transition::Next(task, Ok(next_state)) => {
                        pair = (task, next_state);
                    }
                    Transition::Next(task, Err(err)) => {
                        let (task, next_state) = next_state.transition.fallback(task, err).await;
                        pair = (task, next_state);
                    }
                    Transition::Crashed(err) => {
                        break;
                    }
                    Transition::Interrupted => {
                        break;
                    }
                }
            }
        }
    }
}

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
