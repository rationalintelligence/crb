use crate::attach::ForwardTo;
use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use crb_agent::{Address, Agent, AgentSession, Context, DoAsync, MessageFor, Next, RunAgent};
use crb_core::{Slot, Tag};
use crb_send::{Recipient, Sender};
use derive_more::From;
use futures::channel::oneshot::{self, Canceled};
use futures::{
    task::{Context as FutContext, Poll},
    Future,
};
use std::future::IntoFuture;
use std::pin::Pin;
use thiserror::Error;

pub struct Interplay<IN, OUT> {
    pub request: IN,
    pub responder: Responder<OUT>,
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

impl<IN, OUT> Interplay<IN, OUT> {
    pub fn new_pair(request: IN) -> (Self, Fetcher<OUT>) {
        let (tx, rx) = oneshot::channel();
        let responder = Responder { tx };
        let interplay = Interplay { request, responder };
        let fetcher = Fetcher { rx };
        (interplay, fetcher)
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
}

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("Request failed: {0}")]
    Failed(#[from] anyhow::Error),
    #[error("Request canceled: {0}")]
    Canceled(#[from] Canceled),
}

pub type Output<R> = Result<R, FetchError>;

impl<OUT> Future for Fetcher<OUT> {
    type Output = Output<OUT>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut FutContext<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|result| {
            result
                .map_err(FetchError::from)
                .and_then(|res| res.map_err(FetchError::from))
        })
    }
}

impl<A, OUT, T> ForwardTo<A, T> for Fetcher<OUT>
where
    A: OnResponse<OUT, T>,
    OUT: Tag,
    T: Tag,
{
    type Runtime = RunAgent<FetcherTask<OUT, T>>;

    fn into_trackable(self, address: Address<A>, tag: T) -> Self::Runtime {
        let task = FetcherTask {
            recipient: address.sender(),
            fetcher: self,
            tag: Slot::filled(tag),
        };
        RunAgent::new(task)
    }
}

pub struct FetcherTask<OUT, T> {
    recipient: Recipient<Response<OUT, T>>,
    fetcher: Fetcher<OUT>,
    tag: Slot<T>,
}

impl<OUT, T> Agent for FetcherTask<OUT, T>
where
    OUT: Tag,
    T: Tag,
{
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

#[async_trait]
impl<OUT, T> DoAsync for FetcherTask<OUT, T>
where
    OUT: Tag,
    T: Tag,
{
    async fn once(&mut self, _: &mut ()) -> Result<Next<Self>> {
        let response = (&mut self.fetcher).await;
        self.recipient.send(Response {
            response,
            tag: self.tag.take()?,
        })?;
        Ok(Next::done())
    }
}

impl<OUT, T> IntoFuture for FetcherTask<OUT, T> {
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
        ctx: &mut Context<Self>,
    ) -> Result<()>;
}

struct Response<OUT, T> {
    response: Output<OUT>,
    tag: T,
}

#[async_trait]
impl<A, OUT, T> MessageFor<A> for Response<OUT, T>
where
    A: OnResponse<OUT, T>,
    OUT: Tag,
    T: Tag,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        agent.on_response(self.response, self.tag, ctx).await
    }
}
