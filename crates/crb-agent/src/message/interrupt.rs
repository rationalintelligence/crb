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
        self.interrupt_with_level(1);
    }

    fn interrupt_with_level(&self, level: u8) {
        match level {
            0 => {
                // 0 - Interrupts an actor
                Address::interrupt(&self).ok();
            }
            1 => {
                // 1 - Interrupts a state-machine
                self.stopper().stop(false);
            }
            _ => {
                // 2 - Interrupts an async routine
                self.stopper().stop(true);
            }
        }
    }
}
