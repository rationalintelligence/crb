use crate::agent::Agent;
use crate::context::{AgentContext, Context};
use crate::global::CRB;
use crate::performers::{AgentState, Next, StatePerformer, Transition, TransitionCommand};
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_core::time::Instant;
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
    async fn handle(&mut self, mut state: S, ctx: &mut Context<Self>) -> Result<Next<Self>> {
        let stopper = ctx.session().controller.stopper.clone();
        while stopper.is_active() {
            let iteration = Instant::now();

            let result = self.repeat(&mut state).await;
            match result {
                Ok(Some(state)) => {
                    return Ok(state);
                }
                Ok(None) => {}
                Err(err) => {
                    self.repair(err).await?;
                }
            }

            if iteration.elapsed().as_millis() as usize >= CRB.get_long_threshold() {
                use std::any::type_name;
                log::warn!(
                    "DoAsync<{}> for {} is too long!",
                    type_name::<S>(),
                    type_name::<Self>()
                );
            }
        }
        Ok(Next::interrupt())
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

    async fn fallback_with_context(&mut self, err: Error, _ctx: &mut Context<Self>) -> Next<Self> {
        self.fallback(err).await
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
        let state = self.state.take().unwrap();
        let next_state = match agent.handle(state, ctx).await {
            Ok(next) => next,
            Err(err) => agent.fallback_with_context(err, ctx).await,
        };
        let command = TransitionCommand::Next(next_state);
        Transition::Continue { agent, command }
    }
}
