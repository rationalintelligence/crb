use super::{Fetcher, Interplay};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::{Address, Agent, Context, MessageFor};

pub trait InteractExt<R: Request> {
    fn interact(&self, request: R) -> Fetcher<R::Response>;
}

impl<A, R> InteractExt<R> for Address<A>
where
    A: OnRequest<R>,
    R: Request,
{
    fn interact(&self, request: R) -> Fetcher<R::Response> {
        let (interplay, fetcher) = Interplay::new_pair(request);
        let msg = Interaction { interplay };
        let res = self.send(msg);
        let fetcher = fetcher.grasp(res);
        fetcher
    }
}

impl<A, R> InteractExt<R> for Context<A>
where
    A: OnRequest<R>,
    R: Request,
{
    fn interact(&self, request: R) -> Fetcher<R::Response> {
        self.address().interact(request)
    }
}

pub trait Request: Send + 'static {
    type Response: Send + 'static;
}

pub struct Interaction<R: Request> {
    pub interplay: Interplay<R, R::Response>,
}

#[async_trait]
impl<A, R> MessageFor<A> for Interaction<R>
where
    A: OnRequest<R>,
    R: Request,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        agent.handle(*self, ctx).await
    }
}

#[async_trait]
pub trait OnRequest<R: Request>: Agent {
    async fn handle(&mut self, msg: Interaction<R>, ctx: &mut Context<Self>) -> Result<()> {
        let resp = self.on_request(msg.interplay.request, ctx).await;
        msg.interplay.responder.send_result(resp)
    }

    async fn on_request(&mut self, _request: R, _ctx: &mut Context<Self>) -> Result<R::Response> {
        Err(anyhow!("The on_request method in not implemented."))
    }
}
