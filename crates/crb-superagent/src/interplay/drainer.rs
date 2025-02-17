use crate::supervisor::ForwardTo;
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Address, Agent, AgentSession, DoAsync, Next, OnEvent, RunAgent};
use crb_core::{
    time::{timeout, Duration},
    Msg, Tag,
};
use crb_runtime::InterruptionLevel;
use crb_send::{Recipient, Sender};
use futures::{
    stream::BoxStream,
    task::{Context, Poll},
    Stream, StreamExt,
};
use std::pin::{pin, Pin};

pub struct Drainer<ITEM> {
    stream: BoxStream<'static, ITEM>,
}

impl<ITEM> Drainer<ITEM>
where
    ITEM: Msg,
{
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = ITEM> + Send + 'static,
    {
        Self {
            stream: stream.boxed(),
        }
    }
}

impl<ITEM> Stream for Drainer<ITEM> {
    type Item = ITEM;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        pin!(&mut self.get_mut().stream).poll_next(cx)
    }
}

impl<A, ITEM, T> ForwardTo<A, T> for Drainer<ITEM>
where
    A: OnEvent<ITEM, T>,
    ITEM: Msg,
    T: Tag + Sync + Clone,
{
    type Runtime = RunAgent<DrainerTask<ITEM>>;

    fn into_trackable(self, address: Address<A>, tag: T) -> Self::Runtime {
        let task = DrainerTask {
            recipient: address.recipient_tagged(tag),
            stream: self.stream,
        };
        let mut runtime = RunAgent::new(task);
        runtime.level = InterruptionLevel::ABORT;
        runtime
    }
}

pub struct DrainerTask<ITEM> {
    recipient: Recipient<ITEM>,
    stream: Pin<Box<dyn Stream<Item = ITEM> + Send>>,
}

impl<ITEM> Agent for DrainerTask<ITEM>
where
    ITEM: Msg,
{
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

#[async_trait]
impl<ITEM> DoAsync for DrainerTask<ITEM>
where
    ITEM: Msg,
{
    async fn repeat(&mut self, _: &mut ()) -> Result<Option<Next<Self>>> {
        let duration = Duration::from_secs(5);
        match timeout(duration, self.stream.next()).await {
            Ok(Some(item)) => {
                // The next item forwarding
                self.recipient.send(item)?;
                Ok(None)
            }
            Ok(None) => {
                // Stream is ended
                Ok(Some(Next::done()))
            }
            Err(_) => {
                // Timeout, try again
                Ok(None)
            }
        }
    }
}
