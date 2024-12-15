use crate::message::MessageFor;
use crate::runtime::Address;
use crate::Actor;
use anyhow::Error;
use async_trait::async_trait;

impl<T: Actor> Address<T> {
    pub fn interrupt(&self) -> Result<(), Error> {
        self.send(Interrupt)
    }
}

struct Interrupt;

#[async_trait]
impl<T: Actor> MessageFor<T> for Interrupt {
    async fn handle(self: Box<Self>, actor: &mut T, ctx: &mut T::Context) -> Result<(), Error> {
        actor.interrupt(ctx).await
    }
}
