use crate::interplay::{Fetcher, ManageSubscription, StateEntry, SubscribeExt, Subscription};
use anyhow::Result;
use crb_agent::{Address, OnEvent, TheEvent};

pub struct UniAddress<P: ActorProtocol> {
    address: Box<dyn AbstractAddress<P>>,
}

pub trait AbstractAddress<P> {}

pub trait ActorProtocol {}

pub trait EventHandler<E> {
    fn event(&self, event: E) -> Result<()>;
}

impl<A, E> EventHandler<E> for Address<A>
where
    A: OnEvent<E>,
    E: TheEvent,
{
    fn event(&self, event: E) -> Result<()> {
        Address::event(self, event)
    }
}

pub trait SubscriptionHandler<S: Subscription> {
    fn subscribe(&self, sub: S) -> Fetcher<StateEntry<S>>;
}

impl<A, S> SubscriptionHandler<S> for Address<A>
where
    A: ManageSubscription<S>,
    S: Subscription,
{
    fn subscribe(&self, sub: S) -> Fetcher<StateEntry<S>> {
        SubscribeExt::subscribe(self, sub)
    }
}
