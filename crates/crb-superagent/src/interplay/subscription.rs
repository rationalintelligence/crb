use super::Interplay;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::Agent;

pub trait Subscription: Send + 'static {
    type Entry: Send + 'static;
}

#[async_trait]
pub trait SubscribeTo<S: Subscription>: Agent {
    async fn handle(&mut self, msg: Subscribe<S>, ctx: &mut Self::Context) -> Result<()> {
        let resp = self.on_subscribe(msg.interplay.request, ctx).await;
        msg.interplay.responder.send_result(resp)
    }

    async fn on_subscribe(&mut self, _sub: S, _ctx: &mut Self::Context) -> Result<S::Entry> {
        Err(anyhow!("The on_subscribe method in not implemented."))
    }
}

pub trait UnsubscribeFrom<S>: Agent {}

pub struct Subscribe<S: Subscription> {
    pub interplay: Interplay<S, S::Entry>,
}
