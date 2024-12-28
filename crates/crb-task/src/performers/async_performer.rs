use crate::hybryd_task::{
    HybrydSession, HybrydState, HybrydTask, NextState, StatePerformer, Transition,
};
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::kit::Interruptor;
use std::marker::PhantomData;

impl<T> NextState<T>
where
    T: HybrydTask,
{
    pub fn do_async<S>(state: S) -> Self
    where
        T: AsyncActivity<S>,
        S: HybrydState,
    {
        let performer = AsyncPerformer {
            _task: PhantomData,
            state: Some(state),
        };
        Self::new(performer)
    }
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
