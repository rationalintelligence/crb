use crate::address::{Address, MessageFor};
use crate::agent::Agent;
use crate::context::Context;
use anyhow::Result;
use async_trait::async_trait;
use crb_runtime::Interruptor;

impl<A: Agent> Address<A> {
    pub fn interrupt(&self) -> Result<()> {
        self.send(Interrupt)
    }
}

struct Interrupt;

#[async_trait]
impl<A: Agent> MessageFor<A> for Interrupt {
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        let name = std::any::type_name::<A>();
        log::trace!("Interrupting agent: {name}");
        agent.interrupt(ctx);
        Ok(())
    }
}

impl<A: Agent> Interruptor for Address<A> {
    fn interrupt(&self) {
        Address::interrupt(self).ok();
    }
}
