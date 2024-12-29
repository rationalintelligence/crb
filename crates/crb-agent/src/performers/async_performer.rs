use crate::agent::{
    RunAgent, AgentSession, AgentState, Agent, NextState, StatePerformer, Transition,
};
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::kit::Interruptor;
use futures::Future;
use std::marker::PhantomData;

impl<T> NextState<T>
where
    T: Agent,
{
    pub fn do_async<S>(state: S) -> Self
    where
        T: AsyncActivity<S>,
        S: AgentState,
    {
        let performer = AsyncPerformer {
            _task: PhantomData,
            state: Some(state),
        };
        Self::new(performer)
    }
}

#[async_trait]
pub trait AsyncActivity<S: Send + 'static>: Agent {
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

    async fn repair(&mut self, err: Error) -> Result<(), Error> {
        Err(err)
    }

    async fn fallback(&mut self, err: Error) -> NextState<Self> {
        NextState::fail(err)
    }
}

struct AsyncPerformer<T, S> {
    _task: PhantomData<T>,
    state: Option<S>,
}

#[async_trait]
impl<T, S> StatePerformer<T> for AsyncPerformer<T, S>
where
    T: AsyncActivity<S>,
    S: AgentState,
{
    async fn perform(&mut self, mut task: T, session: &mut AgentSession<T>) -> Transition<T> {
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

impl RunAgent<AsyncFn> {
    pub fn new_async<F: AnyAsyncFn>(fut: F) -> Self {
        let task = AsyncFn {
            fut: Some(Box::new(fut)),
        };
        Self::new(task)
    }
}

pub trait AnyAsyncFn: Future<Output = Result<()>> + Send + 'static {}

impl<F> AnyAsyncFn for F where F: Future<Output = Result<()>> + Send + 'static {}

struct AsyncFn {
    fut: Option<Box<dyn AnyAsyncFn>>,
}

impl Agent for AsyncFn {
    fn initial_state(&mut self) -> NextState<Self> {
        NextState::do_async(CallFn)
    }
}

struct CallFn;

#[async_trait]
impl AsyncActivity<CallFn> for AsyncFn {
    async fn once(&mut self, _state: &mut CallFn) -> Result<NextState<Self>> {
        let fut = self.fut.take().unwrap();
        let pinned_fut = Box::into_pin(fut);
        pinned_fut.await?;
        Ok(NextState::done())
    }
}
