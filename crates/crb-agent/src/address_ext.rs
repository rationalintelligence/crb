use crate::address::Address;
use crate::agent::Agent;
use crate::context::Context;
use crate::message::event::OnEvent;
use crb_send::Recipient;
use derive_more::{Deref, DerefMut};

pub trait ToAddress<A: Agent> {
    fn to_address(&self) -> Address<A>;
}

impl<A: Agent> ToAddress<A> for Address<A> {
    fn to_address(&self) -> Address<A> {
        self.clone()
    }
}

impl<A: Agent> ToAddress<A> for Context<A> {
    fn to_address(&self) -> Address<A> {
        self.address().clone()
    }
}

impl<A: Agent, T: ToAddress<A>> ToAddress<A> for &T {
    fn to_address(&self) -> Address<A> {
        (**self).to_address().clone()
    }
}

impl<A: Agent, T: ToAddress<A>> ToAddress<A> for &mut T {
    fn to_address(&self) -> Address<A> {
        (**self).to_address().clone()
    }
}

pub trait ToRecipient<M> {
    fn to_recipient(&self) -> Recipient<M>;
}

impl<A, M> ToRecipient<M> for Address<A>
where
    A: OnEvent<M>,
    M: Send + 'static,
{
    fn to_recipient(&self) -> Recipient<M> {
        self.recipient()
    }
}

impl<A, M> ToRecipient<M> for Context<A>
where
    A: OnEvent<M>,
    M: Send + 'static,
{
    fn to_recipient(&self) -> Recipient<M> {
        self.address().recipient()
    }
}

impl<T, M> ToRecipient<M> for &T
where
    T: ToRecipient<M>,
    M: Send + 'static,
{
    fn to_recipient(&self) -> Recipient<M> {
        (**self).to_recipient()
    }
}

impl<T, M> ToRecipient<M> for &mut T
where
    T: ToRecipient<M>,
    M: Send + 'static,
{
    fn to_recipient(&self) -> Recipient<M> {
        (**self).to_recipient()
    }
}

pub trait Equip<A: Agent> {
    fn equip<E>(self) -> E
    where
        E: From<Address<A>>;
}

impl<A> Equip<A> for Address<A>
where
    A: Agent,
{
    fn equip<E>(self) -> E
    where
        E: From<Address<A>>,
    {
        E::from(self)
    }
}

/// This implementation is useful for using with supervisors that can return
/// addresses of spawned agents with assigned relations in a tuple.
impl<A, X> Equip<A> for (Address<A>, X)
where
    A: Agent,
{
    fn equip<E>(self) -> E
    where
        E: From<Address<A>>,
    {
        E::from(self.0)
    }
}

#[derive(Deref, DerefMut)]
pub struct StopAddress<A: Agent> {
    address: Address<A>,
}

impl<A: Agent> From<Address<A>> for StopAddress<A> {
    fn from(address: Address<A>) -> Self {
        Self { address }
    }
}

impl<A: Agent> Drop for StopAddress<A> {
    fn drop(&mut self) {
        self.address.interrupt().ok();
    }
}
