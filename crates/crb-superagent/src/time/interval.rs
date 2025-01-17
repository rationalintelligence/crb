use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, DoAsync, Next, OnEvent, RunAgent, ToAddress};
use crb_core::{
    time::{sleep, Duration},
    Tag,
};
use crb_runtime::{JobHandle, Task};
use crb_send::{Recipient, Sender};

pub struct Interval {
    #[allow(unused)]
    job: JobHandle,
}

impl Interval {
    pub fn new<A, T>(address: impl ToAddress<A>, duration: Duration, event: T) -> Self
    where
        A: OnEvent<T>,
        T: Tag + Clone,
    {
        let task = IntervalTask {
            duration,
            event,
            sender: address.to_address().recipient(),
        };
        let mut job = RunAgent::new(task).spawn().job();
        job.cancel_on_drop(true);
        Self { job }
    }
}

struct IntervalTask<T> {
    duration: Duration,
    event: T,
    sender: Recipient<T>,
}

impl<T> Agent for IntervalTask<T>
where
    T: Tag + Clone,
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
    T: Tag + Clone,
{
    async fn repeat(&mut self, _: &mut ()) -> Result<Option<Next<Self>>> {
        let event = self.event.clone();
        self.sender.send(event)?;
        sleep(self.duration).await;
        Ok(None)
    }
}
