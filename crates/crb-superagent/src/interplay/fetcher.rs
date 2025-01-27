use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use crb_agent::{Agent, Context, MessageFor, ToAddress};
use crb_core::Tag;
use derive_more::{Deref, DerefMut, From, Into};
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

    pub fn forwardable(self) -> FetcherTask<OUT> {
        self.into()
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

#[derive(Deref, DerefMut, From, Into)]
pub struct FetcherTask<OUT> {
    pub fetcher: Fetcher<OUT>,
}

impl<OUT> FetcherTask<OUT> {
    pub fn forward_to<A, T>(self, recipient: impl ToAddress<A>, tag: T)
    where
        A: OnResponse<OUT, T>,
        OUT: Send + 'static,
        T: Tag,
    {
        let address = recipient.to_address();
        crb_core::spawn(async move {
            let response = self.fetcher.await;
            if let Err(err) = address.send(Response { response, tag }) {
                log::error!("Can't send a reponse: {err}");
            }
        });
    }
}

impl<OUT> IntoFuture for FetcherTask<OUT> {
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
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        agent.on_response(self.response, self.tag, ctx).await
    }
}
