use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, Context, DoAsync, MessageFor, Next, RunAgent, ToAddress};
use crb_core::{
    time::{sleep, Duration},
    Slot, SyncTag,
};
use crb_runtime::{JobHandle, Task};
use crb_send::{MessageSender, Sender};

// TODO: Refactor to use `OnEvent`
#[async_trait]
pub trait OnTimeout<T = ()>: Agent {
    async fn on_timeout(&mut self, tag: T, ctx: &mut Context<Self>) -> Result<()>;
}

pub struct Timeout {
    #[allow(unused)]
    job: JobHandle,
}

impl Timeout {
    pub fn new<A, T>(address: impl ToAddress<A>, duration: Duration, tag: T) -> Self
    where
        A: OnTimeout<T>,
        T: SyncTag,
    {
        let task = TimeoutTask {
            duration,
            tag: Slot::filled(tag),
            sender: address.to_address().sender(),
        };
        let mut job = RunAgent::new(task).spawn().job();
        job.cancel_on_drop(true);
        Self { job }
    }
}

struct TimeoutTask<T> {
    duration: Duration,
    tag: Slot<T>,
    sender: MessageSender<Completed<T>>,
}

impl<T> Agent for TimeoutTask<T>
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
impl<T> DoAsync for TimeoutTask<T>
where
    T: SyncTag,
{
    async fn once(&mut self, _: &mut ()) -> Result<Next<Self>> {
        sleep(self.duration).await;
        let tick = Completed {
            tag: self.tag.take()?,
        };
        self.sender.send(tick)?;
        Ok(Next::done())
    }
}

struct Completed<T> {
    tag: T,
}

#[async_trait]
impl<A, T> MessageFor<A> for Completed<T>
where
    A: OnTimeout<T>,
    T: SyncTag,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        agent.on_timeout(self.tag, ctx).await
    }
}
