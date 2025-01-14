use anyhow::{anyhow, Error, Result};
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

pub struct Responder<OUT> {
    tx: oneshot::Sender<Result<OUT>>,
}

impl<OUT> Responder<OUT> {
    pub fn send(self, resp: OUT) -> Result<()> {
        self.send_result(Ok(resp))
    }

    pub fn send_result(self, resp: Result<OUT>) -> Result<()> {
        self.tx
            .send(resp)
            .map_err(|_| anyhow!("Can't send the response."))
    }
}

pub struct Interplay<IN, OUT> {
    pub request: IN,
    pub responder: Responder<OUT>,
}

pub struct Interaction<R: Request> {
    pub interplay: Interplay<R, R::Response>,
}

impl<R: Request> Interaction<R> {
    pub fn new_pair(request: R) -> (Self, Fetcher<R::Response>) {
        let (tx, rx) = oneshot::channel();
        let responder = Responder { tx };
        let interplay = Interplay { request, responder };
        let interaction = Interaction { interplay };
        let fetcher = Fetcher { rx };
        (interaction, fetcher)
    }
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

#[must_use]
pub struct Fetcher<OUT> {
    rx: oneshot::Receiver<Result<OUT>>,
}

impl<OUT> Fetcher<OUT> {
    pub fn grasp(self, result: Result<()>) -> Self {
        match result {
            Ok(_) => self,
            Err(err) => Self::spoiled(err),
        }
    }

    pub fn spoiled(err: Error) -> Fetcher<OUT> {
        let (tx, rx) = oneshot::channel();
        tx.send(Err(err)).ok();
        Fetcher { rx }
    }

    pub fn forward_to<A, T>(self, address: Address<A>, tag: T)
    where
        A: OnResponse<OUT, T>,
        OUT: Send + 'static,
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

impl<OUT> Future for Fetcher<OUT> {
    type Output = Output<OUT>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut FutContext<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|result| {
            result
                .map_err(ResponseError::from)
                .and_then(|res| res.map_err(ResponseError::from))
        })
    }
}

pub trait AddressExt<R: Request> {
    fn interact(&self, request: R) -> Fetcher<R::Response>;
}

impl<A, R> AddressExt<R> for Address<A>
where
    A: OnRequest<R>,
    R: Request,
{
    fn interact(&self, request: R) -> Fetcher<R::Response> {
        let (msg, fetcher) = Interaction::new_pair(request);
        let res = self.send(msg);
        fetcher.grasp(res)
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

pub type Output<R> = Result<R, ResponseError>;

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
