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
            listeners: vec![address.to_address().recipient()],
        };
        let mut job = RunAgent::new(task).spawn().job();
        job.cancel_on_drop(true);
        Self { job }
    }
}

pub struct IntervalSwitch<T> {
    job: Option<JobHandle>,
    task: IntervalTask<T>,
}

impl<T> IntervalSwitch<T>
where
    T: Tag + Clone,
{
    pub fn new(duration: Duration, event: T) -> Self {
        let task = IntervalTask {
            duration,
            event,
            listeners: Vec::new(),
        };
        Self { job: None, task }
    }

    pub fn on(&mut self) {
        if self.job.is_none() {
            let task = self.task.clone();
            let mut job = RunAgent::new(task).spawn().job();
            job.cancel_on_drop(true);
            self.job = Some(job);
        }
    }

    pub fn off(&mut self) {
        self.job.take();
    }

    pub fn add_listener<A>(&mut self, address: impl ToAddress<A>)
    where
        A: OnEvent<T>,
    {
        let recipient = address.to_address().recipient();
        self.task.listeners.push(recipient);
    }
}

#[derive(Clone)]
struct IntervalTask<T> {
    duration: Duration,
    event: T,
    listeners: Vec<Recipient<T>>,
}

impl<T> Agent for IntervalTask<T>
where
    T: Tag + Clone,
{
    type Context = AgentSession<Self>;

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
        for listener in &self.listeners {
            listener.send(self.event.clone())?;
        }
        sleep(self.duration).await;
        Ok(None)
    }
}
