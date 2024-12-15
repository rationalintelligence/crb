pub mod message;
pub mod context;
pub mod runtime;

use context::Address;
use anyhow::Error;
use async_trait::async_trait;
use crb_runtime::context::ManagedContext;
use crb_runtime::interruptor::Interruptor;
use crb_runtime::context::Context;
use runtime::ActorRuntime;
use context::ActorContext;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::DerefMut;

#[async_trait]
pub trait Actor: Sized + Send + 'static {
    type Context: ManagedContext + DerefMut<Target = ActorContext<Self>>;
    type GroupBy: Debug + Ord + Clone + Sync + Send + Eq + Hash;

    async fn initialize(&mut self, _ctx: &mut Self::Context) -> Result<(), Error> {
        Ok(())
    }

    async fn interrupt(&mut self, ctx: &mut Self::Context) -> Result<(), Error> {
        // Closes the channel
        ctx.shutdown();
        Ok(())
    }

    async fn event(&mut self, ctx: &mut Self::Context) -> Result<(), Error> {
        if let Some(envelope) = ctx.next_envelope().await {
            envelope.handle(self, ctx).await?;
        } else {
            // Terminates the runtime when the channel has drained
            ctx.controller().stop(false)?;
        }
        Ok(())
    }

    async fn finalize(&mut self, _ctx: &mut Self::Context) -> Result<(), Error> {
        Ok(())
    }
}

pub trait Standalone: Actor {
    fn spawn(self) -> Address<Self>
    where Self::Context: From<ActorContext<Self>>;
}

impl<T: Actor + 'static> Standalone for T {
    fn spawn(self) -> Address<Self>
    where Self::Context: From<ActorContext<Self>> {
        let context = ActorContext::new();
        let address = context.address().clone();
        let context = T::Context::from(context);
        let runtime = ActorRuntime { actor: self, context };
        crb_core::spawn(runtime.entrypoint());
        address
    }
}
