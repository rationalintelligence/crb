use crate::address::MessageFor;
use crate::agent::Agent;
use crate::context::AgentContext;
use crate::performers::Next;
use anyhow::{Error, Result};
use async_trait::async_trait;

#[async_trait]
pub trait InContext<E>: Agent {
    async fn handle(&mut self, event: E, ctx: &mut Self::Context) -> Result<Next<Self>>;

    async fn fallback(&mut self, err: Error, _ctx: &mut Self::Context) -> Next<Self> {
        Next::fail(err)
    }
}

pub struct LoopbackEvent<E> {
    event: E,
}

impl<E> LoopbackEvent<E> {
    pub fn new(event: E) -> Self {
        Self { event }
    }
}

#[async_trait]
impl<A, E> MessageFor<A> for LoopbackEvent<E>
where
    A: InContext<E>,
    E: Send + 'static,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut A::Context) -> Result<()> {
        let next_state = match agent.handle(self.event, ctx).await {
            Ok(next) => next,
            Err(err) => agent.fallback(err, ctx).await,
        };
        ctx.session().do_next(next_state);
        Ok(())
    }
}
