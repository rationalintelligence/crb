use crate::agent::Agent;
use crate::context::Context;
use crate::performers::{Next, StatePerformer, Transition, TransitionCommand};
use anyhow::{Error, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Duty<E>: Agent {
    async fn handle(&mut self, event: E, ctx: &mut Context<Self>) -> Result<Next<Self>>;

    async fn fallback(&mut self, err: Error, _ctx: &mut Context<Self>) -> Next<Self> {
        Next::fail(err)
    }
}

impl<A> Next<A>
where
    A: Agent,
{
    pub fn duty<E>(event: E) -> Self
    where
        A: Duty<E>,
        E: Send + 'static,
    {
        Self::new(DutyPerformer { event: Some(event) })
    }
}

pub struct DutyPerformer<E> {
    event: Option<E>,
}

#[async_trait]
impl<A, E> StatePerformer<A> for DutyPerformer<E>
where
    A: Duty<E>,
    E: Send + 'static,
{
    async fn perform(&mut self, mut agent: A, ctx: &mut Context<A>) -> Transition<A> {
        let event = self.event.take().unwrap();
        let next_state = match agent.handle(event, ctx).await {
            Ok(next) => next,
            Err(err) => agent.fallback(err, ctx).await,
        };
        let command = TransitionCommand::Next(next_state);
        Transition::Continue { agent, command }
    }
}
