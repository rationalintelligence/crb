pub mod message;
pub mod context;
pub mod runtime;

use anyhow::Error;
use async_trait::async_trait;
use crb_runtime::context::BasicContext;
use context::ActorContext;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::DerefMut;

#[async_trait]
pub trait Actor: Sized + Send + 'static {
    type Context: BasicContext + DerefMut<Target = ActorContext<Self>>;
    type GroupBy: Debug + Ord + Clone + Sync + Send + Eq + Hash;

    async fn initialize(&mut self, _ctx: &mut Self::Context) -> Result<(), Error> {
        Ok(())
    }

    async fn interrupt(&mut self, ctx: &mut Self::Context) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }

    async fn event(&mut self, ctx: &mut Self::Context) -> Result<(), Error> {
        if let Some(envelope) = ctx.next_envelope().await {
            envelope.handle(self, ctx).await?;
        } else {
            // ctx.stop();
        }
        Ok(())
    }

    async fn finalize(&mut self, _ctx: &mut Self::Context) -> Result<(), Error> {
        Ok(())
    }
}
