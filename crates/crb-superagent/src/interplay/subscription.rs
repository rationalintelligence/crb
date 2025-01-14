use super::{Fetcher, Interplay};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::{Address, Agent, MessageFor};
use crb_core::UniqueId;
use crb_runtime::Context;
use crb_send::{Recipient, Sender};

pub trait SubscribeExt<S: Subscription> {
    fn subscribe(&self, request: S) -> Fetcher<StateEntry<S>>;
}

impl<A, S> SubscribeExt<S> for Address<A>
where
    A: ManageSubscription<S>,
    S: Subscription,
{
    fn subscribe(&self, subscription: S) -> Fetcher<StateEntry<S>> {
        let sub_id = UniqueId::new(subscription);
        let (interplay, fetcher) = Interplay::new_pair(sub_id);
        let msg = Subscribe { interplay };
        let res = self.send(msg);
        let fetcher = fetcher.grasp(res);
        fetcher
    }
}

pub struct StateEntry<S: Subscription> {
    pub state: S::State,
    pub entry: Entry<S>,
}

pub struct Entry<S: Subscription> {
    sub_id: UniqueId<S>,
    recipient: Recipient<Unsubscribe<S>>,
}

impl<S: Subscription> Drop for Entry<S> {
    fn drop(&mut self) {
        let msg = Unsubscribe {
            sub_id: self.sub_id.clone(),
        };
        self.recipient.send(msg).ok();
    }
}

pub trait Subscription: Sync + Send + 'static {
    type State: Send + 'static;
}

#[async_trait]
pub trait ManageSubscription<S: Subscription>: Agent {
    async fn handle(&mut self, msg: Subscribe<S>, ctx: &mut Self::Context) -> Result<()> {
        let sub_id = msg.interplay.request;
        let res = self.subscribe(sub_id.clone(), ctx).await;
        let state_entry = match res {
            Ok(state) => {
                let recipient = ctx.address().sender();
                let entry = Entry { sub_id, recipient };
                let state_entry = StateEntry { state, entry };
                Ok(state_entry)
            }
            Err(err) => Err(err),
        };
        msg.interplay.responder.send_result(state_entry)
    }

    async fn subscribe(
        &mut self,
        sub_id: UniqueId<S>,
        _ctx: &mut Self::Context,
    ) -> Result<S::State> {
        Err(anyhow!(
            "The on_subscribe method in not implemented to handle {sub_id}."
        ))
    }

    async fn unsubscribe(&mut self, sub_id: UniqueId<S>, _ctx: &mut Self::Context) -> Result<()> {
        Err(anyhow!(
            "The on_unsubscribe method in not implemented to handle {sub_id}"
        ))
    }
}

pub struct Subscribe<S: Subscription> {
    pub interplay: Interplay<UniqueId<S>, StateEntry<S>>,
}

#[async_trait]
impl<A, S> MessageFor<A> for Subscribe<S>
where
    A: ManageSubscription<S>,
    S: Subscription,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut A::Context) -> Result<()> {
        agent.handle(*self, ctx).await
    }
}

pub struct Unsubscribe<S: Subscription> {
    pub sub_id: UniqueId<S>,
}

#[async_trait]
impl<A, S> MessageFor<A> for Unsubscribe<S>
where
    A: ManageSubscription<S>,
    S: Subscription,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut A::Context) -> Result<()> {
        agent.unsubscribe(self.sub_id, ctx).await
    }
}
