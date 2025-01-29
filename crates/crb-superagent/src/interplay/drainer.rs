use crate::supervisor::ForwardTo;
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Address, Agent, AgentSession, Context, DoAsync, MessageFor, Next, RunAgent};
use crb_core::{
    time::{timeout, Duration},
    Msg, Tag,
};
use crb_runtime::InterruptionLevel;
use crb_send::{Recipient, Sender};
use futures::{Stream, StreamExt};
use std::pin::Pin;

pub struct Drainer<ITEM> {
    stream: Pin<Box<dyn Stream<Item = ITEM> + Send>>,
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
            stream: Box::pin(stream),
        }
    }
}

impl<A, ITEM, T> ForwardTo<A, T> for Drainer<ITEM>
where
    A: OnItem<ITEM, T>,
    ITEM: Msg,
    T: Tag + Clone,
{
    type Runtime = RunAgent<DrainerTask<ITEM, T>>;

    fn into_trackable(self, address: Address<A>, tag: T) -> Self::Runtime {
        let task = DrainerTask {
            recipient: address.sender(),
            stream: self.stream,
            tag,
        };
        let mut runtime = RunAgent::new(task);
        runtime.level = InterruptionLevel::ABORT;
        runtime
    }
}

pub struct DrainerTask<ITEM, T> {
    recipient: Recipient<Item<ITEM, T>>,
    stream: Pin<Box<dyn Stream<Item = ITEM> + Send>>,
    tag: T,
}

impl<ITEM, T> Agent for DrainerTask<ITEM, T>
where
    ITEM: Msg,
    T: Tag + Clone,
{
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

#[async_trait]
impl<ITEM, T> DoAsync for DrainerTask<ITEM, T>
where
    ITEM: Msg,
    T: Tag + Clone,
{
    async fn repeat(&mut self, _: &mut ()) -> Result<Option<Next<Self>>> {
        let duration = Duration::from_secs(5);
        match timeout(Some(duration), self.stream.next()).await {
            Ok(Some(item)) => {
                // The next item forwarding
                let item = Item {
                    item,
                    tag: self.tag.clone(),
                };
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

#[async_trait]
pub trait OnItem<ITEM, T = ()>: Agent {
    async fn on_item(&mut self, item: ITEM, tag: T, ctx: &mut Context<Self>) -> Result<()>;
}

struct Item<ITEM, T> {
    item: ITEM,
    tag: T,
}

#[async_trait]
impl<A, ITEM, T> MessageFor<A> for Item<ITEM, T>
where
    A: OnItem<ITEM, T>,
    ITEM: Msg,
    T: Tag,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        agent.on_item(self.item, self.tag, ctx).await
    }
}
