use crate::attach::ForwardTo;
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Address, Agent, AgentSession, Context, DoAsync, MessageFor, Next, RunAgent};
use crb_core::Tag;
use crb_send::{Recipient, Sender};
use futures::{Stream, StreamExt};
use std::pin::Pin;

pub struct Drainer<T> {
    stream: Pin<Box<dyn Stream<Item = T> + Send>>,
}

impl<T> Drainer<T>
where
    T: Tag,
{
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = T> + Send + 'static,
    {
        Self {
            stream: Box::pin(stream),
        }
    }
}

impl<A, T> ForwardTo<A> for Drainer<T>
where
    A: OnItem<T>,
    T: Tag,
{
    type Runtime = RunAgent<DrainerTask<T>>;

    fn into_trackable(self, address: Address<A>) -> Self::Runtime {
        let task = DrainerTask {
            recipient: address.sender(),
            stream: self.stream,
        };
        RunAgent::new(task)
    }
}

pub struct DrainerTask<T> {
    recipient: Recipient<Item<T>>,
    stream: Pin<Box<dyn Stream<Item = T> + Send>>,
}

impl<T> Agent for DrainerTask<T>
where
    T: Tag,
{
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

#[async_trait]
impl<T> DoAsync for DrainerTask<T>
where
    T: Tag,
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
pub trait OnItem<OUT, T = ()>: Agent {
    async fn on_item(&mut self, item: OUT, tag: T, ctx: &mut Context<Self>) -> Result<()>;
}

struct Item<OUT, T = ()> {
    item: OUT,
    tag: T,
}

#[async_trait]
impl<A, OUT, T> MessageFor<A> for Item<OUT, T>
where
    A: OnItem<OUT, T>,
    OUT: Send + 'static,
    T: Tag,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        agent.on_item(self.item, self.tag, ctx).await
    }
}
