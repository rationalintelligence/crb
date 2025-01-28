use crate::agent::Agent;
use crate::address::Address;
use crb_runtime::{Stopper, Interruptor};

pub struct AgentInterruptor<A: Agent> {
    address: Address<A>,
    stopper: Stopper,
}

impl<A: Agent> AgentInterruptor<A> {
    pub fn new(address: Address<A>, stopper: Stopper) -> Self {
        Self {
            address,
            stopper,
        }
    }
}

impl<A: Agent> Interruptor for AgentInterruptor<A> {
    fn interrupt(&self) {
        Address::interrupt(&self.address).ok();
        self.stopper.stop(false);
    }
}
