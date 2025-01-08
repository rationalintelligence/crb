use crate::agent::Agent;
use crate::performers::{Next, StatePerformer, Transition, TransitionCommand};
use anyhow::{Error, Result};
use async_trait::async_trait;

#[async_trait]
pub trait InContext<E>: Agent {
    async fn handle(&mut self, event: E, ctx: &mut Self::Context) -> Result<Next<Self>>;

    async fn fallback(&mut self, err: Error, _ctx: &mut Self::Context) -> Next<Self> {
        Next::fail(err)
    }
}

impl<A> Next<A>
where
    A: Agent,
{
    pub fn in_context<E>(event: E) -> Self
    where
        A: InContext<E>,
        E: Send + 'static,
    {
        Self::new(InContextPerformer { event: Some(event) })
    }
}

pub struct InContextPerformer<E> {
    event: Option<E>,
}

#[async_trait]
impl<A, E> StatePerformer<A> for InContextPerformer<E>
where
    A: InContext<E>,
    E: Send + 'static,
{
    async fn perform(&mut self, mut agent: A, ctx: &mut A::Context) -> Transition<A> {
        let event = self.event.take().unwrap();
        let next_state = match agent.handle(event, ctx).await {
            Ok(next) => next,
            Err(err) => agent.fallback(err, ctx).await,
        };
        let command = TransitionCommand::Next(next_state);
        Transition::Continue { agent, command }
    }
}
