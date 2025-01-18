use crate::agent::Agent;
use crate::context::{AgentContext, Context};
use crate::performers::{
    AgentState, ConsumptionReason, Next, StatePerformer, Transition, TransitionCommand,
};
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::Interruptor;
use std::marker::PhantomData;
use tokio::task::spawn_blocking;

impl<T> Next<T>
where
    T: Agent,
{
    pub fn do_sync<S>(state: S) -> Self
    where
        T: DoSync<S>,
        S: AgentState,
    {
        let performer = SyncPerformer {
            _task: PhantomData,
            state: Some(state),
        };
        Self::new(performer)
    }
}

pub trait DoSync<S = ()>: Agent {
    fn perform(&mut self, mut state: S, interruptor: Interruptor) -> Next<Self> {
        while interruptor.is_active() {
            let result = self.repeat(&mut state);
            match result {
                Ok(Some(state)) => {
                    return state;
                }
                Ok(None) => {}
                Err(err) => {
                    if let Err(err) = self.repair(err) {
                        return self.fallback(err);
                    }
                }
            }
        }
        Next::interrupt()
    }

    fn repeat(&mut self, state: &mut S) -> Result<Option<Next<Self>>> {
        self.once(state).map(Some)
    }

    fn once(&mut self, _state: &mut S) -> Result<Next<Self>> {
        Ok(Next::done())
    }

    fn repair(&mut self, err: Error) -> Result<(), Error> {
        Err(err)
    }

    fn fallback(&mut self, err: Error) -> Next<Self> {
        Next::fail(err)
    }
}

struct SyncPerformer<T, S> {
    _task: PhantomData<T>,
    state: Option<S>,
}

#[async_trait]
impl<T, S> StatePerformer<T> for SyncPerformer<T, S>
where
    T: DoSync<S>,
    S: AgentState,
{
    async fn perform(&mut self, mut agent: T, ctx: &mut Context<T>) -> Transition<T> {
        let interruptor = ctx.session().controller.interruptor.clone();
        let state = self.state.take().unwrap();
        let handle = spawn_blocking(move || {
            let next_state = agent.perform(state, interruptor);
            let command = TransitionCommand::Next(next_state);
            Transition::Continue { agent, command }
        });
        match handle.await {
            Ok(transition) => transition,
            Err(err) => {
                let err = err.into();
                let reason = ConsumptionReason::Crashed(err);
                Transition::Consume { reason }
            }
        }
    }
}
