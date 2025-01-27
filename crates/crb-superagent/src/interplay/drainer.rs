use crate::attach::ForwardTo;
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Address, Agent, AgentSession, Context, DoAsync, MessageFor, Next, RunAgent};
use crb_core::Tag;
use crb_send::{Recipient, Sender};
use futures::{Stream, StreamExt};
use std::pin::Pin;

pub struct Drainer<ITEM> {
    stream: Pin<Box<dyn Stream<Item = ITEM> + Send>>,
}

impl<ITEM> Drainer<ITEM>
where
    ITEM: Tag,
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
    A: OnItem<ITEM>,
    ITEM: Tag,
    T: Tag,
{
    type Runtime = RunAgent<DrainerTask<ITEM>>;

    fn into_trackable(self, address: Address<A>, tag: T) -> Self::Runtime {
        let task = DrainerTask {
            recipient: address.sender(),
            stream: self.stream,
        };
        RunAgent::new(task)
    }
}

pub struct DrainerTask<ITEM> {
    recipient: Recipient<Item<ITEM>>,
    stream: Pin<Box<dyn Stream<Item = ITEM> + Send>>,
}

impl<ITEM> Agent for DrainerTask<ITEM>
where
    ITEM: Tag,
{
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

#[async_trait]
impl<ITEM> DoAsync for DrainerTask<ITEM>
where
    ITEM: Tag,
{
    async fn repeat(&mut self, _: &mut ()) -> Result<Option<Next<Self>>> {
        if let Some(item) = self.stream.next().await {
            let item = Item { item, tag: () };
            self.recipient.send(item)?;
            Ok(None)
        } else {
            Ok(Some(Next::done()))
        }
    }
}

#[async_trait]
pub trait OnItem<OUT, ITEM = ()>: Agent {
    async fn on_item(&mut self, item: OUT, tag: ITEM, ctx: &mut Context<Self>) -> Result<()>;
}

struct Item<OUT, ITEM = ()> {
    item: OUT,
    tag: ITEM,
}

#[async_trait]
impl<A, OUT, ITEM> MessageFor<A> for Item<OUT, ITEM>
where
    A: OnItem<OUT, ITEM>,
    OUT: Tag,
    ITEM: Tag,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        agent.on_item(self.item, self.tag, ctx).await
    }
}
