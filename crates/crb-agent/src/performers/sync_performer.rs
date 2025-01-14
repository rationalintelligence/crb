use crate::agent::{Agent, Output};
use crate::context::{AgentContext, AgentSession, Context};
use crate::performers::{
    AgentState, ConsumptionReason, Next, StatePerformer, Transition, TransitionCommand,
};
use crate::runtime::RunAgent;
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
    async fn perform(&mut self, mut agent: T, ctx: &mut T::Context) -> Transition<T> {
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

impl<T: Output> RunAgent<SyncFn<T>> {
    pub fn new_sync<F: AnySyncFn<T>>(func: F) -> Self {
        let task = SyncFn::<T> {
            func: Some(Box::new(func)),
            output: None,
        };
        Self::new(task)
    }
}

pub trait AnySyncFn<T>: FnOnce() -> T + Send + 'static {}

impl<F, T> AnySyncFn<T> for F where F: FnOnce() -> T + Send + 'static {}

pub struct SyncFn<T> {
    func: Option<Box<dyn AnySyncFn<T>>>,
    output: Option<T>,
}

impl<T: Output> Agent for SyncFn<T> {
    type Context = AgentSession<Self>;
    type Output = T;

    fn initialize(&mut self, _ctx: &mut Context<Self>) -> Next<Self> {
        Next::do_sync(CallFn)
    }

    fn finalize(self, _ctx: &mut Context<Self>) -> Option<Self::Output> {
        self.output
    }
}

struct CallFn;

impl<T: Output> DoSync<CallFn> for SyncFn<T> {
    fn once(&mut self, _state: &mut CallFn) -> Result<Next<Self>> {
        let func = self.func.take().unwrap();
        let output = func();
        self.output = Some(output);
        Ok(Next::done())
    }
}
