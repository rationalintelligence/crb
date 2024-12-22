use crate::actor::Actor;
use crate::message::MessageFor;
use crate::runtime::Address;
use anyhow::Result;
use async_trait::async_trait;

impl<A: Actor> Address<A> {
    pub fn interrupt(&self) -> Result<()> {
        self.send(Interrupt)
    }
}

struct Interrupt;

#[async_trait]
impl<A: Actor> MessageFor<A> for Interrupt {
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut A::Context) -> Result<()> {
        actor.interrupt(ctx).await
    }
}
