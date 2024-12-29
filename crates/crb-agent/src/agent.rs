use crate::context::AgentContext;
use crb_runtime::context::ManagedContext;
use crate::runtime::Next;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Agent: Sized + Send + 'static {
    type Output: Default + Clone + Sync + Send;
    type Context: AgentContext<Self>;

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

    fn finalize(self, _ctx: &mut Self::Context) -> Self::Output {
        Self::Output::default()
    }
}
