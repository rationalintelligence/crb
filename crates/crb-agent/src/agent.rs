use crate::context::{AgentContext, Context};
use crate::performers::Next;
use crate::runtime::RunAgent;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::{InteractiveTask, ManagedContext, ReachableContext};
use std::any::type_name;

#[async_trait]
pub trait Agent: Sized + Send + 'static {
    type Context: AgentContext<Self>;
    type Output: Output;

    fn initialize(&mut self, _ctx: &mut Context<Self>) -> Next<Self> {
        self.begin()
    }

    fn begin(&mut self) -> Next<Self> {
        Next::events()
    }

    fn interrupt(&mut self, ctx: &mut Context<Self>) {
        ctx.shutdown();
    }

    async fn event(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        let envelope = ctx.session().joint().next_envelope();
        if let Some(envelope) = envelope.await {
            envelope.handle(self, ctx).await?;
        } else {
            // Terminates the runtime when the channel has drained
            ctx.stop();
        }
        Ok(())
    }

    fn failed(&mut self, err: &Error, _ctx: &mut Context<Self>) {
        log::error!("Agent [{}] failed: {err}", type_name::<Self>());
    }

    fn finalize(self, _ctx: &mut Context<Self>) -> Option<Self::Output> {
        self.end()
    }

    fn end(self) -> Option<Self::Output> {
        None
    }
}

pub trait Output: Sync + Send + 'static {}

impl<T> Output for T where T: Sync + Send + 'static {}

pub trait Standalone: Agent {
    fn spawn(self) -> <Self::Context as ReachableContext>::Address
    where
        Self::Context: Default,
    {
        RunAgent::new(self).spawn_connected()
    }
    // TODO: spawn_with_context()
}

#[async_trait]
pub trait Runnable: Agent {
    async fn run(self) -> Result<Option<Self::Output>>;
}

#[async_trait]
impl<A: Agent> Runnable for A
where
    Self::Context: Default,
    A::Output: Clone,
{
    async fn run(self) -> Result<Option<Self::Output>> {
        let mut runtime = RunAgent::new(self);
        runtime.perform_routine().await?;
        let output = runtime.context.address().clone().join().await?.output();
        Ok(output)
    }
}
