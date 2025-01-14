use super::{Fetcher, Interplay, Output};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::{Address, Agent, MessageFor};
use crb_core::Tag;
use derive_more::{Deref, DerefMut, From, Into};
use std::future::IntoFuture;

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

// TODO: Add the `Ctx` wrapper and implement for that as well

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

#[derive(Deref, DerefMut, From, Into)]
pub struct ForwardableFetcher<OUT> {
    pub fetcher: Fetcher<OUT>,
}

impl<OUT> ForwardableFetcher<OUT> {
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

impl<OUT> IntoFuture for ForwardableFetcher<OUT> {
    type Output = Output<OUT>;
    type IntoFuture = Fetcher<OUT>;

    fn into_future(self) -> Self::IntoFuture {
        self.fetcher
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
