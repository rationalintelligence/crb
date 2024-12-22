use crate::message::MessageFor;
use crate::runtime::Address;
use crate::Actor;
use anyhow::Error;
use async_trait::async_trait;

impl<A: Actor> Address<A> {
    pub fn interrupt(&self) -> Result<(), Error> {
        self.send(Interrupt)
    }
}

struct Interrupt;

#[async_trait]
impl<A: Actor> MessageFor<A> for Interrupt {
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut A::Context) -> Result<(), Error> {
        actor.interrupt(ctx).await
    }
}
