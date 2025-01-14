use super::{Output, Interplay, Fetcher};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::{Address, Agent, MessageFor};
use crb_core::Tag;

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
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut A::Context) -> Result<()> {
        agent.handle(*self, ctx).await
    }
}

#[async_trait]
pub trait OnRequest<R: Request>: Agent {
    async fn handle(&mut self, msg: Interaction<R>, ctx: &mut Self::Context) -> Result<()> {
        let resp = self.on_request(msg.interplay.request, ctx).await;
        msg.interplay.responder.send_result(resp)
    }

    async fn on_request(&mut self, _request: R, _ctx: &mut Self::Context) -> Result<R::Response> {
        Err(anyhow!("The on_request method in not implemented."))
    }
}

pub struct ResponseFetcher<OUT> {
    pub fetcher: Fetcher<OUT>,
}

impl<OUT> ResponseFetcher<OUT> {
    pub fn forward_to<A, T>(self, address: Address<A>, tag: T)
    where
        A: OnResponse<OUT, T>,
        OUT: Send + 'static,
        T: Tag,
    {
        crb_core::spawn(async move {
            let response = self.fetcher.await;
            if let Err(err) = address.send(Response { response, tag }) {
                log::error!("Can't send a reponse: {err}");
            }
        });
    }
}

pub trait AddressExt<R: Request> {
    fn interact(&self, request: R) -> ResponseFetcher<R::Response>;
}

impl<A, R> AddressExt<R> for Address<A>
where
    A: OnRequest<R>,
    R: Request,
{
    fn interact(&self, request: R) -> ResponseFetcher<R::Response> {
        let (interplay, fetcher) = Interplay::new_pair(request);
        let msg = Interaction { interplay };
        let res = self.send(msg);
        let fetcher = fetcher.grasp(res);
        ResponseFetcher { fetcher }
    }
}

#[async_trait]
pub trait OnResponse<OUT, T = ()>: Agent {
    async fn on_response(
        &mut self,
        response: Output<OUT>,
        tag: T,
        ctx: &mut Self::Context,
    ) -> Result<()>;
}

struct Response<OUT, T = ()> {
    response: Output<OUT>,
    tag: T,
}

#[async_trait]
impl<A, OUT, T> MessageFor<A> for Response<OUT, T>
where
    A: OnResponse<OUT, T>,
    OUT: Send + 'static,
    T: Tag,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut A::Context) -> Result<()> {
        agent.on_response(self.response, self.tag, ctx).await
    }
}
