use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, DoAsync, Next, OnEvent, RunAgent, ToAddress};
use crb_core::{
    time::{sleep, Duration},
    Slot, SyncTag,
};
use crb_runtime::{JobHandle, Task};
use crb_send::{Recipient, Sender};

pub struct Timeout {
    #[allow(unused)]
    job: JobHandle,
}

impl Timeout {
    pub fn new<A, T>(address: impl ToAddress<A>, duration: Duration, event: T) -> Self
    where
        A: OnEvent<T>,
        T: SyncTag,
    {
        let task = TimeoutTask {
            duration,
            event: Slot::filled(event),
            sender: address.to_address().recipient(),
        };
        let mut job = RunAgent::new(task).spawn().job();
        job.cancel_on_drop(true);
        Self { job }
    }
}

struct TimeoutTask<T> {
    duration: Duration,
    event: Slot<T>,
    sender: Recipient<T>,
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
        let event = self.event.take()?;
        self.sender.send(event)?;
        Ok(Next::done())
    }
}
