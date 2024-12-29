use crate::context::AgentContext;
use crate::runtime::RunAgent;
use crate::performers::Next;
use anyhow::Result;
use async_trait::async_trait;
use crb_runtime::kit::{Context, InteractiveTask, ManagedContext};

#[async_trait]
pub trait Agent: Sized + Send + 'static {
    type Context: AgentContext<Self>;
    type Output: Output;

    fn initialize(&mut self, _ctx: &mut Self::Context) -> Next<Self> {
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

    fn finalize(&mut self, _ctx: &mut Self::Context) -> Self::Output {
        Self::Output::default()
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
