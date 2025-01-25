use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, DoAsync, Next, OnEvent, RunAgent, ToAddress};
use crb_core::{
    time::{sleep, Duration},
    Tag,
};
use crb_runtime::{JobHandle, Task};
use crb_send::{Recipient, Sender};

// TODO: Add TimerBuilder

pub struct TimerHandle {
    #[allow(unused)]
    job: Option<JobHandle>,
}

pub struct Timer<T> {
    job: Option<JobHandle>,
    task: TimerTask<T>,
}

impl<T> Timer<T>
where
    T: Tag + Clone,
{
    pub fn just_spawn<A>(address: impl ToAddress<A>, duration: Duration, event: T) -> TimerHandle
    where
        A: OnEvent<T>,
    {
        let mut switch = Self::new(event);
        switch.set_duration(duration);
        switch.add_listener(address);
        switch.start();
        TimerHandle {
            job: switch.job.take(),
        }
    }

    pub fn new(event: T) -> Self {
        let task = TimerTask {
            duration: Duration::from_secs(1),
            event,
            listeners: Vec::new(),
            repeat: false,
        };
        Self { job: None, task }
    }

    pub fn is_active(&self) -> bool {
        self.job.is_some()
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.task.duration = duration;
    }

    pub fn set_repeat(&mut self, repeat: bool) {
        self.task.repeat = repeat;
    }

    pub fn restart(&mut self) {
        self.stop();
        self.start();
    }

    pub fn start(&mut self) {
        if !self.is_active() {
            let task = self.task.clone();
            let mut job = RunAgent::new(task).spawn().job();
            job.cancel_on_drop(true);
            self.job = Some(job);
        }
    }

    pub fn stop(&mut self) {
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
    repeat: bool,
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

impl<T> TimerTask<T>
where
    T: Tag + Clone,
{
    fn distribute(&self) {
        for listener in &self.listeners {
            listener.send(self.event.clone()).ok();
        }
    }
}

#[async_trait]
impl<T> DoAsync for TimerTask<T>
where
    T: Tag + Clone,
{
    async fn repeat(&mut self, _: &mut ()) -> Result<Option<Next<Self>>> {
        if self.repeat {
            self.distribute();
            sleep(self.duration).await;
            Ok(None)
        } else {
            sleep(self.duration).await;
            self.distribute();
            Ok(Some(Next::done()))
        }
    }
}
