use anyhow::Error;
use async_trait::async_trait;
use crb_actor::kit::{Actor, Address, MessageFor};
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
    Failed(#[from] Error),
    #[error("Request canceled: {0}")]
    Canceled(#[from] Canceled),
}

pub trait Request: Send + 'static {
    type Response: Send + 'static;
}

pub struct Interaction<T: Request> {
    request: T,
    tx: oneshot::Sender<Result<T::Response, Error>>,
}

#[async_trait]
impl<A, T> MessageFor<A> for Interaction<T>
where
    A: OnRequest<T>,
    T: Request,
{
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut A::Context) -> Result<(), Error> {
        actor.handle(*self, ctx).await
    }
}

#[async_trait]
pub trait OnRequest<T: Request>: Actor {
    async fn handle(&mut self, msg: Interaction<T>, ctx: &mut Self::Context) -> Result<(), Error> {
        let resp = self.on_request(msg.request, ctx).await;
        msg.tx
            .send(resp)
            .map_err(|_| Error::msg("Can't send the response"))
    }

    async fn on_request(
        &mut self,
        request: T,
        ctx: &mut Self::Context,
    ) -> Result<T::Response, Error>;
}

#[must_use]
pub struct Responder<T: Request> {
    rx: oneshot::Receiver<Result<T::Response, Error>>,
}

impl<T: Request> Responder<T> {
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

impl<T: Request> Future for Responder<T> {
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
    fn interact(&self, request: T) -> Result<Responder<T>, Error>;
}

impl<A, T> AddressExt<T> for Address<A>
where
    A: OnRequest<T>,
    T: Request,
{
    fn interact(&self, request: T) -> Result<Responder<T>, Error> {
        let (tx, rx) = oneshot::channel();
        let interaction = Interaction { request, tx };
        self.send(interaction)?;
        Ok(Responder { rx })
    }
}

#[async_trait]
pub trait OnResponse<T: Request>: Actor {
    async fn on_response(
        &mut self,
        response: Output<T::Response>,
        ctx: &mut Self::Context,
    ) -> Result<(), Error>;
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
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut A::Context) -> Result<(), Error> {
        actor.on_response(self.response, ctx).await
    }
}
