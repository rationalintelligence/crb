use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::{Address, Agent, MessageFor};
use crb_core::Tag;
use futures::{
    channel::oneshot::{self, Canceled},
    task::{Context as FutContext, Poll},
    Future,
};
use std::pin::Pin;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResponseError {
    #[error("Request failed: {0}")]
    Failed(#[from] anyhow::Error),
    #[error("Request canceled: {0}")]
    Canceled(#[from] Canceled),
}

pub trait Request: Send + 'static {
    type Response: Send + 'static;
}

pub struct Responder<R: Request> {
    tx: oneshot::Sender<Result<R::Response>>,
}

impl<R: Request> Responder<R> {
    pub fn send(self, resp: R::Response) -> Result<()> {
        self.send_result(Ok(resp))
    }

    pub fn send_result(self, resp: Result<R::Response>) -> Result<()> {
        self.tx
            .send(resp)
            .map_err(|_| anyhow!("Can't send the response."))
    }
}

pub struct Interaction<R: Request> {
    pub request: R,
    pub responder: Responder<R>,
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
        let resp = self.on_request(msg.request, ctx).await;
        msg.responder.send_result(resp)
    }

    async fn on_request(&mut self, _request: R, _ctx: &mut Self::Context) -> Result<R::Response> {
        Err(anyhow!("The on_request method in not implemented."))
    }
}

#[must_use]
pub struct Fetcher<R: Request> {
    rx: oneshot::Receiver<Result<R::Response>>,
}

impl<R: Request> Fetcher<R> {
    pub fn forward_to<A, T>(self, address: Address<A>, tag: T)
    where
        A: OnResponse<R, T>,
        T: Tag,
    {
        crb_core::spawn(async move {
            let response = self.await;
            if let Err(err) = address.send(Response { response, tag }) {
                log::error!("Can't send a reponse: {err}");
            }
        });
    }
}

impl<R: Request> Future for Fetcher<R> {
    type Output = Output<R::Response>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut FutContext<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|result| {
            result
                .map_err(ResponseError::from)
                .and_then(|res| res.map_err(ResponseError::from))
        })
    }
}

pub trait AddressExt<R: Request> {
    fn interact(&self, request: R) -> Fetcher<R>;
}

impl<A, R> AddressExt<R> for Address<A>
where
    A: OnRequest<R>,
    R: Request,
{
    fn interact(&self, request: R) -> Fetcher<R> {
        let (tx, rx) = oneshot::channel();
        let responder = Responder { tx };
        let interaction = Interaction { request, responder };
        if let Err(err) = self.send(interaction) {
            // Report about sending error in the responder itseld
            let (tx, rx) = oneshot::channel();
            // TODO: Consider alternative implementation to reuse the same channel
            // Add an extra trait to restore a value from `MessageFor`,
            // but that could be more expensive
            tx.send(Err(err)).ok();
            Fetcher { rx }
        } else {
            Fetcher { rx }
        }
    }
}

#[async_trait]
pub trait OnResponse<R: Request, T = ()>: Agent {
    async fn on_response(
        &mut self,
        response: Output<R::Response>,
        tag: T,
        ctx: &mut Self::Context,
    ) -> Result<()>;
}

pub type Output<R> = Result<R, ResponseError>;

struct Response<R: Request, T = ()> {
    response: Output<R::Response>,
    tag: T,
}

#[async_trait]
impl<A, R, T> MessageFor<A> for Response<R, T>
where
    A: OnResponse<R, T>,
    R: Request,
    T: Tag,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut A::Context) -> Result<()> {
        agent.on_response(self.response, self.tag, ctx).await
    }
}
