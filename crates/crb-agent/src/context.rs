use crate::address::{Address, AddressJoint};
use crate::agent::Agent;
use crate::performers::Next;
use crb_runtime::{Context, Controller, ManagedContext};
use derive_more::{Deref, DerefMut};

pub trait AgentContext<A: Agent + ?Sized>
where
    Self: Context<Address = Address<A>>,
    Self: ManagedContext,
{
    // TODO: Replace with explicit methods
    fn session(&mut self) -> &mut AgentSession<A>;
}

#[derive(Deref, DerefMut)]
pub struct AgentSession<A: Agent + ?Sized> {
    pub controller: Controller,
    pub next_state: Option<Next<A>>,
    pub joint: AddressJoint<A>,
    #[deref]
    #[deref_mut]
    pub address: Address<A>,
}

impl<A: Agent> AgentSession<A> {
    pub fn joint(&mut self) -> &mut AddressJoint<A> {
        &mut self.joint
    }

    pub fn do_next(&mut self, next_state: Next<A>) {
        self.next_state = Some(next_state);
    }
}

impl<A: Agent> Default for AgentSession<A> {
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

impl<A: Agent> Context for AgentSession<A> {
    type Address = Address<A>;

    fn address(&self) -> &Self::Address {
        &self.address
    }
}

impl<A: Agent> AsRef<Address<A>> for AgentSession<A> {
    fn as_ref(&self) -> &Address<A> {
        self.address()
    }
}

impl<A: Agent> ManagedContext for AgentSession<A> {
    fn is_alive(&self) -> bool {
        self.controller.is_active()
    }

    fn shutdown(&mut self) {
        self.joint.close();
    }

    fn stop(&mut self) {
        self.controller.stop(false);
    }
}

impl<A: Agent> AgentContext<A> for AgentSession<A> {
    fn session(&mut self) -> &mut AgentSession<A> {
        self
    }
}
