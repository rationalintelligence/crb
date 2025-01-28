use crate::address::Address;
use crate::agent::Agent;
use crb_runtime::{Interruptor, Stopper};

pub struct AgentInterruptor<A: Agent> {
    address: Address<A>,
    stopper: Stopper,
}

impl<A: Agent> AgentInterruptor<A> {
    pub fn new(address: Address<A>, stopper: Stopper) -> Self {
        Self { address, stopper }
    }
}

impl<A: Agent> Interruptor for AgentInterruptor<A> {
    fn interrupt(&self) {
        Address::interrupt(&self.address).ok();
        self.stopper.stop(false);
    }

    fn interrupt_with_level(&self, level: u8) {
        match level {
            0 => {
                Address::interrupt(&self.address).ok();
            }
            1 => {
                self.stopper.stop(false);
            }
            _ => {
                self.stopper.stop(true);
            }
        }
    }
}
