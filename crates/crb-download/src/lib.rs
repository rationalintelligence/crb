pub mod progress;

use anyhow::Error;
use bytes::Bytes;
use derive_more::Deref;
use futures::{
    task::{Context, Poll},
    Stream,
};
use progress::ProgressCalc;
use reqwest::{Body, Response, StatusCode};
use std::pin::Pin;

#[derive(Deref)]
pub struct Downloader {
    #[deref]
    progress_calc: ProgressCalc,
    status: StatusCode,
    stream: Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>>>>,
}

impl Downloader {
    pub async fn download(url: &str) -> Result<Self, Error> {
        let resp = reqwest::get(url).await?;
        Ok(Self::from_response(resp))
    }

    pub fn from_response(response: Response) -> Self {
        let total = response.content_length();
        Self {
            progress_calc: ProgressCalc::new(total),
            status: response.status(),
            stream: Box::pin(response.bytes_stream()),
        }
    }
}

impl Stream for Downloader {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = Pin::new(&mut self.stream);
        stream
            .poll_next(cx)
            .map_ok(|chunk| {
                let len = chunk.len() as u64;
                self.progress_calc.inc(len);
                chunk
            })
            .map_err(Error::from)
    }
}
