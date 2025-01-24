use crate::interplay::{
    Fetcher, InteractExt, ManageSubscription, OnRequest, Request, StateEntry, SubscribeExt,
    Subscription,
};
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

pub trait InteractionHandler<R: Request> {
    fn interact(&self, req: R) -> Fetcher<R::Response>;
}

impl<A, R> InteractionHandler<R> for Address<A>
where
    A: OnRequest<R>,
    R: Request,
{
    fn interact(&self, req: R) -> Fetcher<R::Response> {
        InteractExt::interact(self, req)
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
