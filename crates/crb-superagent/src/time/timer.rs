use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, DoAsync, Next, OnEvent, RunAgent, ToAddress};
use crb_core::{
    time::{sleep, Duration},
    Tag,
};
use crb_runtime::{JobHandle, Task};
use crb_send::{Recipient, Sender};

pub struct Timer {
    #[allow(unused)]
    job: Option<JobHandle>,
}

impl Timer {
    pub fn new<A, T>(address: impl ToAddress<A>, duration: Duration, event: T) -> Self
    where
        A: OnEvent<T>,
        T: Tag + Clone,
    {
        let mut switch = TimerSwitch::new(duration, event);
        switch.add_listener(address);
        switch.start();
        Self {
            job: switch.job.take(),
        }
    }
}

pub struct TimerSwitch<T> {
    job: Option<JobHandle>,
    task: TimerTask<T>,
}

impl<T> TimerSwitch<T>
where
    T: Tag + Clone,
{
    pub fn new(duration: Duration, event: T) -> Self {
        let task = TimerTask {
            duration,
            event,
            listeners: Vec::new(),
        };
        Self { job: None, task }
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.task.duration = duration;
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
struct TimerTask<T> {
    duration: Duration,
    event: T,
    listeners: Vec<Recipient<T>>,
}

impl<T> Agent for TimerTask<T>
where
    T: Tag + Clone,
{
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

#[async_trait]
impl<T> DoAsync for TimerTask<T>
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
