pub mod interaction;
pub mod ping;
pub mod subscription;

pub use interaction::*;
pub use ping::*;
pub use subscription::*;

use anyhow::{anyhow, Error, Result};
use futures::{
    // TODO: Use crb_core?
    channel::oneshot::{self, Canceled},
    task::{Context as FutContext, Poll},
    Future,
};
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

    pub fn forwardable(self) -> ForwardableFetcher<OUT> {
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

impl<IN, OUT> Interplay<IN, OUT> {
    pub fn new_pair(request: IN) -> (Self, Fetcher<OUT>) {
        let (tx, rx) = oneshot::channel();
        let responder = Responder { tx };
        let interplay = Interplay { request, responder };
        let fetcher = Fetcher { rx };
        (interplay, fetcher)
    }
}
