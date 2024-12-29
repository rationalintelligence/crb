use crate::address::{Address, AddressJoint};
use crate::agent::Agent;
use crate::runtime::NextState;
use crb_runtime::kit::{Controller, Context, ManagedContext};

pub trait AgentContext<T: Agent>: Context {
    fn session(&mut self) -> &mut AgentSession<T>;
}

pub struct AgentSession<T: Agent> {
    pub controller: Controller,
    pub next_state: Option<NextState<T>>,
    pub joint: AddressJoint<T>,
    pub address: Address<T>,
}

impl<T: Agent> AgentSession<T> {
    pub fn joint(&mut self) -> &mut AddressJoint<T> {
        &mut self.joint
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
    type Address = ();

    fn address(&self) -> &Self::Address {
        &()
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

