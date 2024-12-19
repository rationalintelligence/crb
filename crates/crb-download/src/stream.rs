use crate::destination::Tempfile;
use crate::progress::ProgressCalc;
use anyhow::{Error, Result};
use bytes::Bytes;
use derive_more::Deref;
use futures::{
    ready,
    task::{Context, Poll},
    Future, Stream, StreamExt,
};
use reqwest::{Client, Response};
use std::pin::Pin;

#[derive(Deref)]
pub struct Downloader {
    #[deref]
    progress: ProgressCalc,
    state: State,
}

impl Downloader {
    pub fn new(url: &str) -> Self {
        let resp = Client::new().get(url).send();
        Self {
            progress: ProgressCalc::new(None),
            state: State::Request(Box::pin(resp)),
        }
    }

    pub async fn download(mut self) -> Result<Tempfile> {
        let mut dest = Tempfile::create().await?;
        while let Some(chunk) = self.next().await {
            dest.write_chunk(&chunk?).await?;
        }
        dest.finalize().await?;
        Ok(dest)
    }
}

impl Stream for Downloader {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.state {
            State::Request(req) => {
                let req = Pin::new(req);
                let response = ready!(req.poll(cx))?.error_for_status()?;
                let total = response.content_length();
                self.progress.change_total(total);
                let stream = Box::pin(response.bytes_stream());
                self.state = State::Stream(stream);
                self.poll_next(cx)
            }
            State::Stream(stream) => {
                let stream = Pin::new(stream);
                stream
                    .poll_next(cx)
                    .map_ok(|chunk| {
                        let len = chunk.len() as u64;
                        self.progress.inc(len);
                        chunk
                    })
                    .map_err(Error::from)
            }
        }
    }
}

enum State {
    Request(Pin<Box<dyn Future<Output = reqwest::Result<Response>>>>),
    Stream(Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>>>>),
}
