use crate::address::Address;
use crate::address::MessageFor;
use crate::agent::Agent;
use anyhow::Result;
use async_trait::async_trait;

impl<A: Agent> Address<A> {
    pub fn interrupt(&self) -> Result<()> {
        self.send(Interrupt)
    }
}

struct Interrupt;

#[async_trait]
impl<A: Agent> MessageFor<A> for Interrupt {
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut A::Context) -> Result<()> {
        actor.interrupt(ctx);
        Ok(())
    }
}
