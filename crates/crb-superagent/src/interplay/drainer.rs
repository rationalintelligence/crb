use crate::supervisor::ForwardTo;
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Address, Agent, AgentSession, Context, DoAsync, MessageFor, Next, RunAgent};
use crb_core::{Msg, Tag};
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
        RunAgent::new(task)
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
        if let Some(item) = self.stream.next().await {
            let item = Item {
                item,
                tag: self.tag.clone(),
            };
            self.recipient.send(item)?;
            Ok(None)
        } else {
            Ok(Some(Next::done()))
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
