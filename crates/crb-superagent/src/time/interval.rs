use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Address, Agent, AgentSession, Context, DoAsync, MessageFor, Next, RunAgent};
use crb_core::{
    time::{sleep, Duration},
    SyncTag,
};
use crb_runtime::{JobHandle, Task};
use crb_send::{MessageSender, Sender};
use std::sync::Arc;

#[async_trait]
pub trait OnTick<T = ()>: Agent {
    async fn on_tick(&mut self, tag: &T, ctx: &mut Context<Self>) -> Result<()>;
}

pub struct Interval {
    #[allow(unused)]
    job: JobHandle,
}

impl Interval {
    pub fn new<A, T>(address: Address<A>, duration: Duration, tag: T) -> Self
    where
        A: OnTick<T>,
        T: SyncTag,
    {
        let task = IntervalTask {
            duration,
            tag: Arc::new(tag),
            sender: address.sender(),
        };
        let mut job = RunAgent::new(task).spawn().job();
        job.cancel_on_drop(true);
        Self { job }
    }
}

struct IntervalTask<T> {
    duration: Duration,
    tag: Arc<T>,
    sender: MessageSender<Tick<T>>,
}

impl<T> Agent for IntervalTask<T>
where
    T: SyncTag,
{
    type Context = AgentSession<Self>;
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

#[async_trait]
impl<T> DoAsync for IntervalTask<T>
where
    T: SyncTag,
{
    async fn repeat(&mut self, _: &mut ()) -> Result<Option<Next<Self>>> {
        let tick = Tick {
            tag: self.tag.clone(),
        };
        self.sender.send(tick)?;
        sleep(self.duration).await;
        Ok(None)
    }
}

struct Tick<T> {
    tag: Arc<T>,
}

#[async_trait]
impl<A, T> MessageFor<A> for Tick<T>
where
    A: OnTick<T>,
    T: SyncTag,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        agent.on_tick(&self.tag, ctx).await
    }
}
