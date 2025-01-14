use crate::address::Address;
use crate::agent::Agent;
use crate::context::Context;

pub trait AddressFor<A: Agent> {
    fn address(&self) -> Address<A>;
}

impl<A: Agent> AddressFor<A> for Address<A> {
    fn address(&self) -> Address<A> {
        self.clone()
    }
}

impl<A: Agent> AddressFor<A> for &mut Address<A> {
    fn address(&self) -> Address<A> {
        (*self).clone()
    }
}

impl<A: Agent> AddressFor<A> for &Address<A> {
    fn address(&self) -> Address<A> {
        (*self).clone()
    }
}

impl<A: Agent> AddressFor<A> for Context<A> {
    fn address(&self) -> Address<A> {
        Context::address(self).clone()
    }
}

impl<A: Agent> AddressFor<A> for &Context<A> {
    fn address(&self) -> Address<A> {
        Context::address(self).clone()
    }
}

impl<A: Agent> AddressFor<A> for &mut Context<A> {
    fn address(&self) -> Address<A> {
        Context::address(self).clone()
    }
}
