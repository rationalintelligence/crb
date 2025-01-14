use super::Interplay;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::Agent;
use std::marker::PhantomData;

pub struct Entry<S> {
    _type: PhantomData<S>,
}

pub trait Subscription: Send + 'static {
    type State: Send + 'static;
}

#[async_trait]
pub trait SubscribeTo<S: Subscription>: Agent {
    async fn handle(&mut self, msg: Subscribe<S>, ctx: &mut Self::Context) -> Result<()> {
        let resp = self.on_subscribe(msg.interplay.request, ctx).await;
        msg.interplay.responder.send_result(resp)
    }

    async fn on_subscribe(&mut self, _sub: S, _ctx: &mut Self::Context) -> Result<S::State> {
        Err(anyhow!("The on_subscribe method in not implemented."))
    }
}

pub trait UnsubscribeFrom<S>: Agent {}

pub struct Subscribe<S: Subscription> {
    pub interplay: Interplay<S, S::State>,
}

pub struct Unsubscribe<S: Subscription> {
    pub entry: Entry<S>,
}
