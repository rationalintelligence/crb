use crate::context::AgentContext;
use crate::performers::Next;
use crate::runtime::RunAgent;
use anyhow::{anyhow as err, Error, Result};
use async_trait::async_trait;
use crb_runtime::{Context, InteractiveTask, ManagedContext};

#[async_trait]
pub trait Agent: Sized + Send + 'static {
    type Context: AgentContext<Self>;
    type Output: Output;

    fn initialize(&mut self, _ctx: &mut Self::Context) -> Next<Self> {
        self.begin()
    }

    fn begin(&mut self) -> Next<Self> {
        Next::process()
    }

    fn interrupt(&mut self, ctx: &mut Self::Context) {
        // Closes the channel
        ctx.session().shutdown();
    }

    async fn event(&mut self, ctx: &mut Self::Context) -> Result<()> {
        let envelope = ctx.session().joint().next_envelope();
        if let Some(envelope) = envelope.await {
            envelope.handle(self, ctx).await?;
        } else {
            // Terminates the runtime when the channel has drained
            ctx.session().controller().stop(false)?;
        }
        Ok(())
    }

    fn failed(&mut self, err: &Error, _ctx: &mut Self::Context) {
        log::error!("Agent failed: {err}");
    }

    fn finalize(&mut self, _ctx: &mut Self::Context) -> Option<Self::Output> {
        self.end()
    }

    fn end(&mut self) -> Option<Self::Output> {
        None
    }
}

pub trait Output: Default + Clone + Sync + Send + 'static {}

impl<T> Output for T where T: Default + Clone + Sync + Send + 'static {}

pub trait Standalone: Agent {
    fn spawn(self) -> <Self::Context as Context>::Address
    where
        Self::Context: Default,
    {
        RunAgent::new(self).spawn_connected()
    }
    // TODO: spawn_with_context()
}

#[async_trait]
pub trait Runnable: Agent {
    async fn run(self) -> Result<Self::Output>;
}

#[async_trait]
impl<A: Agent> Runnable for A
where
    Self::Context: Default,
{
    async fn run(self) -> Result<Self::Output> {
        let output = RunAgent::new(self).perform_routine().await?;
        output.ok_or_else(|| err!("Attempt to get output from the consumed agent"))
    }
}
