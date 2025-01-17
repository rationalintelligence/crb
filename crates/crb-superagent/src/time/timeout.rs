use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, DoAsync, Next, OnEvent, RunAgent, ToAddress};
use crb_core::{
    time::{sleep, Duration},
    Tag,
};
use crb_runtime::{JobHandle, Task};
use crb_send::{Recipient, Sender};

pub struct Timeout<T> {
    job: Option<JobHandle>,
    task: TimeoutTask<T>,
}

impl<T> Timeout<T>
where
    T: Tag + Clone,
{
    pub fn new(duration: Duration, event: T) -> Self {
        let task = TimeoutTask {
            duration,
            event,
            listeners: Vec::new(),
        };
        Self { job: None, task }
    }

    pub fn start(&mut self) {
        self.clear();
        let task = self.task.clone();
        let mut job = RunAgent::new(task).spawn().job();
        job.cancel_on_drop(true);
        self.job = Some(job);
    }

    pub fn clear(&mut self) {
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
struct TimeoutTask<T> {
    duration: Duration,
    event: T,
    listeners: Vec<Recipient<T>>,
}

impl<T> Agent for TimeoutTask<T>
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
impl<T> DoAsync for TimeoutTask<T>
where
    T: Tag + Clone,
{
    async fn once(&mut self, _: &mut ()) -> Result<Next<Self>> {
        sleep(self.duration).await;
        for listener in &self.listeners {
            listener.send(self.event.clone())?;
        }
        Ok(Next::done())
    }
}
