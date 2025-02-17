use crate::address::Address;
use crate::agent::Agent;
use crate::context::Context;
use crate::message::event::{Event, OnEvent, TheEvent};
use anyhow::Result;
use crb_send::{Recipient, Sender};
use derive_more::{Deref, DerefMut};
use std::sync::Arc;

pub type UniAddress<T> = Arc<T>;

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

impl<'a, A> Equip<A> for &'a Context<A>
where
    A: Agent,
{
    fn equip<E>(self) -> E
    where
        E: From<Address<A>>,
    {
        self.to_address().into()
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

impl<A: Agent> Address<A> {
    pub fn to_stop_address(self) -> StopAddress<A> {
        StopAddress { address: self }
    }
}

impl<A: Agent> Drop for StopAddress<A> {
    fn drop(&mut self) {
        self.address.interrupt().ok();
    }
}

impl<A, E> Sender<E> for StopAddress<A>
where
    A: OnEvent<E>,
    E: TheEvent,
{
    fn send(&self, event: E) -> Result<()> {
        self.address.send(Event::new(event))
    }
}

impl<A: Agent> StopAddress<A> {
    pub fn to_stop_recipient<E>(self) -> StopRecipient<E>
    where
        A: OnEvent<E>,
        E: TheEvent,
    {
        let recipient = Recipient::new(self);
        StopRecipient { recipient }
    }
}

#[derive(Deref, DerefMut)]
pub struct StopRecipient<E> {
    recipient: Recipient<E>,
}
