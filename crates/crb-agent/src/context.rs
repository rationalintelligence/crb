use crate::address::{Address, AddressJoint, Envelope};
use crate::agent::Agent;
use crate::extension::ExtensionFor;
use crate::performers::Next;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_runtime::{Controller, ManagedContext, ReachableContext};
use derive_more::{Deref, DerefMut};
use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Deref, DerefMut)]
pub struct Context<A: Agent> {
    #[deref]
    #[deref_mut]
    context: A::Context,
    extensions: Option<HashMap<TypeId, Box<dyn Any + Send>>>,
}

impl<A: Agent> Context<A> {
    pub fn wrap(context: A::Context) -> Self {
        Self {
            context,
            extensions: None,
        }
    }

    pub fn add_extension<E>(&mut self, ext: E)
    where
        E: ExtensionFor<A>,
    {
        self.extensions
            .get_or_insert_default()
            .insert(TypeId::of::<E>(), Box::new(ext));
    }

    pub fn be<E>(&mut self) -> Result<E::View<'_>>
    where
        E: ExtensionFor<A>,
    {
        let type_id = TypeId::of::<E>();
        let ext = self
            .extensions
            .get_or_insert_default()
            .get_mut(&type_id)
            .and_then(|boxed| boxed.downcast_mut::<E>())
            .ok_or_else(|| anyhow!("Extension {:?} is not available.", type_id))?;
        Ok(ext.extend(&mut self.context))
    }
}

impl<A: Agent> Context<A>
where
    A::Context: ReachableContext,
{
    pub fn address(&self) -> &<A::Context as ReachableContext>::Address {
        ReachableContext::address(&self.context)
    }
}

#[async_trait]
pub trait AgentContext<A: Agent>
where
    Self: ReachableContext<Address = Address<A>>,
    Self: ManagedContext,
{
    // TODO: Replace with explicit methods
    fn session(&mut self) -> &mut AgentSession<A>;

    async fn next_envelope(&mut self) -> Option<Envelope<A>>;
}

#[derive(Deref, DerefMut)]
pub struct AgentSession<A: Agent> {
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
        let stopper = controller.stopper.clone();
        let (address, joint) = AddressJoint::new_pair(stopper);
        Self {
            controller,
            next_state: None,
            joint,
            address,
        }
    }
}

impl<A: Agent> ReachableContext for AgentSession<A> {
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

#[async_trait]
impl<A: Agent> AgentContext<A> for AgentSession<A> {
    fn session(&mut self) -> &mut AgentSession<A> {
        self
    }

    async fn next_envelope(&mut self) -> Option<Envelope<A>> {
        self.joint().next_envelope().await
    }
}
