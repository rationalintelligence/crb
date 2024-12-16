pub mod event;
pub mod interrupt;
pub mod message;
pub mod runtime;

pub use event::OnEvent;
pub use runtime::{ActorContext, ActorSession, Standalone};

use anyhow::Error;
use async_trait::async_trait;
use crb_runtime::{Interruptor, ManagedContext};

#[async_trait]
pub trait Actor: Sized + Send + 'static {
    type Context: ActorContext<Self>;

    async fn initialize(&mut self, _ctx: &mut Self::Context) -> Result<(), Error> {
        Ok(())
    }

    async fn interrupt(&mut self, ctx: &mut Self::Context) -> Result<(), Error> {
        // Closes the channel
        ctx.session().shutdown();
        Ok(())
    }

    async fn event(&mut self, ctx: &mut Self::Context) -> Result<(), Error> {
        if let Some(envelope) = ctx.session().next_envelope().await {
            envelope.handle(self, ctx).await?;
        } else {
            // Terminates the runtime when the channel has drained
            ctx.session().controller().stop(false)?;
        }
        Ok(())
    }

    async fn finalize(&mut self, _ctx: &mut Self::Context) -> Result<(), Error> {
        Ok(())
    }
}
