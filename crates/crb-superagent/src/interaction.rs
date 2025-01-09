use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::{Address, Agent, MessageFor};
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
pub struct Fetcher<T: Request> {
    rx: oneshot::Receiver<Result<T::Response>>,
}

impl<T: Request> Fetcher<T> {
    pub fn forward_to<A>(self, address: Address<A>)
    where
        A: OnResponse<T>,
    {
        crb_core::spawn(async move {
            let response = self.await;
            if let Err(err) = address.send(Response { response }) {
                log::error!("Can't send a reponse: {err}");
            }
        });
    }
}

impl<T: Request> Future for Fetcher<T> {
    type Output = Output<T::Response>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut FutContext<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|result| {
            result
                .map_err(ResponseError::from)
                .and_then(|res| res.map_err(ResponseError::from))
        })
    }
}

pub trait AddressExt<T: Request> {
    fn interact(&self, request: T) -> Fetcher<T>;
}

impl<A, T> AddressExt<T> for Address<A>
where
    A: OnRequest<T>,
    T: Request,
{
    fn interact(&self, request: T) -> Fetcher<T> {
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
pub trait OnResponse<T: Request>: Agent {
    async fn on_response(
        &mut self,
        response: Output<T::Response>,
        ctx: &mut Self::Context,
    ) -> Result<()>;
}

type Output<T> = Result<T, ResponseError>;

struct Response<T: Request> {
    response: Output<T::Response>,
}

#[async_trait]
impl<A, T> MessageFor<A> for Response<T>
where
    A: OnResponse<T>,
    T: Request,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut A::Context) -> Result<()> {
        agent.on_response(self.response, ctx).await
    }
}
