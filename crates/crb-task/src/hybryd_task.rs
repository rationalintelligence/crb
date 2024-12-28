use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::kit::{Controller, Failures, Interruptor, Runtime, Task};
use futures::stream::Abortable;
use std::marker::PhantomData;

pub trait HybrydState: Send + 'static {}

impl<T> HybrydState for T where T: Send + 'static {}

pub struct NextState<T: ?Sized> {
    transition: Box<dyn StatePerformer<T>>,
}

impl<T> NextState<T>
where
    T: HybrydTask,
{
    pub(crate) fn new(performer: impl StatePerformer<T>) -> Self {
        Self {
            transition: Box::new(performer),
        }
    }

    pub fn do_async<S>(state: S) -> Self
    where
        T: AsyncActivity<S>,
        S: HybrydState,
    {
        let runner = AsyncPerformer {
            _task: PhantomData,
            state: Some(state),
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
    pub(crate) fn interrupt(error: Option<Error>) -> Self {
        Self {
            transition: Box::new(InterruptPerformer { error }),
        }
    }
}

pub struct InterruptPerformer {
    error: Option<Error>,
}

#[async_trait]
impl<T> StatePerformer<T> for InterruptPerformer
where
    T: HybrydTask,
{
    async fn perform(&mut self, _task: T, _session: &mut HybrydSession) -> Transition<T> {
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

pub enum Transition<T> {
    Next(T, Result<NextState<T>>),
    Crashed(Error),
    Interrupted,
}

#[async_trait]
pub trait StatePerformer<T>: Send + 'static {
    async fn perform(&mut self, task: T, session: &mut HybrydSession) -> Transition<T>;
    async fn fallback(&mut self, task: T, err: Error) -> (T, NextState<T>);
}

struct AsyncPerformer<T, S> {
    _task: PhantomData<T>,
    state: Option<S>,
}

#[async_trait]
impl<T, S> StatePerformer<T> for AsyncPerformer<T, S>
where
    T: AsyncActivity<S>,
    S: HybrydState,
{
    async fn perform(&mut self, mut task: T, session: &mut HybrydSession) -> Transition<T> {
        let interruptor = session.controller.interruptor.clone();
        let state = self.state.take().unwrap();
        let next_state = task.perform(state, interruptor).await;
        Transition::Next(task, next_state)
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
pub trait AsyncActivity<S: Send + 'static>: HybrydTask {
    async fn perform(&mut self, mut state: S, interruptor: Interruptor) -> Result<NextState<Self>> {
        while interruptor.is_active() {
            let result = self.many(&mut state).await;
            match result {
                Ok(Some(state)) => {
                    return Ok(state);
                }
                Ok(None) => {}
                Err(_) => {}
            }
        }
        Ok(NextState::interrupt(None))
    }

    async fn many(&mut self, state: &mut S) -> Result<Option<NextState<Self>>> {
        self.once(state).await.map(Some)
    }

    async fn once(&mut self, _state: &mut S) -> Result<NextState<Self>> {
        Ok(NextState::done())
    }

    async fn fallback(&mut self, err: Error) -> NextState<Self> {
        NextState::fail(err)
    }
}

pub struct HybrydSession {
    pub controller: Controller,
}

pub struct DoHybrid<T> {
    pub task: Option<T>,
    pub session: HybrydSession,
    pub failures: Failures,
}

impl<T: HybrydTask> DoHybrid<T> {
    pub fn new(task: T) -> Self {
        let session = HybrydSession {
            controller: Controller::default(),
        };
        Self {
            task: Some(task),
            session,
            failures: Failures::default(),
        }
    }
}

impl<T: HybrydTask> Task<T> for DoHybrid<T> {}

impl<T: HybrydTask> DoHybrid<T> {
    async fn perform_routine(&mut self) -> Result<(), Error> {
        let reg = self.session.controller.take_registration()?;
        let fut = self.perform_task();
        Abortable::new(fut, reg).await??;
        Ok(())
    }

    async fn perform_task(&mut self) -> Result<(), Error> {
        if let Some(mut task) = self.task.take() {
            let session = &mut self.session;
            let next_state = task.begin().await;
            let mut pair = (task, next_state);
            loop {
                let (task, mut next_state) = pair;
                let res = next_state.transition.perform(task, session).await;
                match res {
                    Transition::Next(task, Ok(next_state)) => {
                        pair = (task, next_state);
                    }
                    Transition::Next(task, Err(err)) => {
                        let (task, next_state) = next_state.transition.fallback(task, err).await;
                        pair = (task, next_state);
                    }
                    Transition::Crashed(err) => {
                        return Err(err);
                    }
                    Transition::Interrupted => {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<T> Runtime for DoHybrid<T>
where
    T: HybrydTask,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.session.controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        let result = self.perform_routine().await;
        self.failures.put(result);
    }
}
