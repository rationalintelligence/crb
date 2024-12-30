use crate::address::{Address, AddressJoint};
use crate::agent::Agent;
use crate::performers::Next;
use crb_runtime::kit::{Context, Controller, ManagedContext};
use derive_more::{Deref, DerefMut};

pub trait AgentContext<T: Agent>: Context<Address = Address<T>> {
    fn session(&mut self) -> &mut AgentSession<T>;
}

#[derive(Deref, DerefMut)]
pub struct AgentSession<T: Agent> {
    pub controller: Controller,
    pub next_state: Option<Next<T>>,
    pub joint: AddressJoint<T>,
    #[deref]
    #[deref_mut]
    pub address: Address<T>,
}

impl<T: Agent> AgentSession<T> {
    pub fn joint(&mut self) -> &mut AddressJoint<T> {
        &mut self.joint
    }

    pub fn do_next(&mut self, next_state: Next<T>) {
        self.next_state = Some(next_state);
    }
}

impl<T: Agent> Default for AgentSession<T> {
    fn default() -> Self {
        let controller = Controller::default();
        let (address, joint) = AddressJoint::new_pair();
        Self {
            controller,
            next_state: None,
            joint,
            address,
        }
    }
}

impl<T: Agent> Context for AgentSession<T> {
    type Address = Address<T>;

    fn address(&self) -> &Self::Address {
        &self.address
    }
}

impl<T: Agent> ManagedContext for AgentSession<T> {
    fn controller(&mut self) -> &mut Controller {
        &mut self.controller
    }

    fn shutdown(&mut self) {
        self.joint.close();
    }
}

impl<T: Agent> AgentContext<T> for AgentSession<T> {
    fn session(&mut self) -> &mut AgentSession<T> {
        self
    }
}
