use crate::actor::Actor;
use anyhow::Error;
use async_trait::async_trait;

pub type Envelope<A> = Box<dyn MessageFor<A>>;

#[async_trait]
pub trait MessageFor<A: Actor + ?Sized>: Send + 'static {
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut A::Context) -> Result<(), Error>;
}
