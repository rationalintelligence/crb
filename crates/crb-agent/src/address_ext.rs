use crate::address::Address;
use crate::agent::Agent;
use crate::context::Context;
use crate::message::event::OnEvent;
use crb_send::Recipient;

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
