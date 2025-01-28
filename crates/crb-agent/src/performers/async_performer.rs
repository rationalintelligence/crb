use crate::agent::Agent;
use crate::context::{AgentContext, Context};
use crate::performers::{AgentState, Next, StatePerformer, Transition, TransitionCommand};
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::Stopper;
use std::marker::PhantomData;

impl<T> Next<T>
where
    T: Agent,
{
    pub fn do_async<S>(state: S) -> Self
    where
        T: DoAsync<S>,
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
pub trait DoAsync<S: Send + 'static = ()>: Agent {
    async fn perform(&mut self, mut state: S, stopper: Stopper) -> Next<Self> {
        while stopper.is_active() {
            let result = self.repeat(&mut state).await;
            match result {
                Ok(Some(state)) => {
                    return state;
                }
                Ok(None) => {}
                Err(err) => {
                    if let Err(err) = self.repair(err).await {
                        return self.fallback(err).await;
                    }
                }
            }
        }
        Next::interrupt()
    }

    async fn repeat(&mut self, state: &mut S) -> Result<Option<Next<Self>>> {
        self.once(state).await.map(Some)
    }

    async fn once(&mut self, _state: &mut S) -> Result<Next<Self>> {
        Ok(Next::done())
    }

    async fn repair(&mut self, err: Error) -> Result<(), Error> {
        Err(err)
    }

    async fn fallback(&mut self, err: Error) -> Next<Self> {
        Next::fail(err)
    }
}

struct AsyncPerformer<T, S> {
    _task: PhantomData<T>,
    state: Option<S>,
}

#[async_trait]
impl<T, S> StatePerformer<T> for AsyncPerformer<T, S>
where
    T: DoAsync<S>,
    S: AgentState,
{
    async fn perform(&mut self, mut agent: T, ctx: &mut Context<T>) -> Transition<T> {
        let stopper = ctx.session().controller.stopper.clone();
        let state = self.state.take().unwrap();
        let next_state = agent.perform(state, stopper).await;
        let command = TransitionCommand::Next(next_state);
        Transition::Continue { agent, command }
    }
}
