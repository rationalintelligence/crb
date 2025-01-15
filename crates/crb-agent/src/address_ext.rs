use crate::address::Address;
use crate::agent::Agent;
use crate::context::Context;

pub trait ToAddress<A: Agent> {
    fn to_address(&self) -> Address<A>;
}

impl<A: Agent> ToAddress<A> for Address<A> {
    fn to_address(&self) -> Address<A> {
        self.clone()
    }
}

impl<A: Agent> ToAddress<A> for &mut Address<A> {
    fn to_address(&self) -> Address<A> {
        (*self).clone()
    }
}

impl<A: Agent> ToAddress<A> for &Address<A> {
    fn to_address(&self) -> Address<A> {
        (*self).clone()
    }
}

impl<A: Agent> ToAddress<A> for Context<A> {
    fn to_address(&self) -> Address<A> {
        self.address().clone()
    }
}

impl<A: Agent> ToAddress<A> for &Context<A> {
    fn to_address(&self) -> Address<A> {
        self.address().clone()
    }
}

impl<A: Agent> ToAddress<A> for &mut Context<A> {
    fn to_address(&self) -> Address<A> {
        self.address().clone()
    }
}
