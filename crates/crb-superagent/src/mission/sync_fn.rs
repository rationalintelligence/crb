use super::{runtime::RunMission, Goal, Mission};
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, Context, DoSync, Next};

impl<T: Goal> RunMission<SyncFn<T>> {
    pub fn new_sync<F: AnySyncFn<T>>(func: F) -> Self {
        let task = SyncFn::<T> {
            func: Some(Box::new(func)),
            output: None,
        };
        Self::new(task)
    }
}

pub trait AnySyncFn<T>: FnOnce() -> T + Send + 'static {}

impl<F, T> AnySyncFn<T> for F where F: FnOnce() -> T + Send + 'static {}

pub struct SyncFn<T> {
    func: Option<Box<dyn AnySyncFn<T>>>,
    output: Option<T>,
}

impl<T: Goal> Agent for SyncFn<T> {
    type Context = AgentSession<Self>;

    fn initialize(&mut self, _ctx: &mut Context<Self>) -> Next<Self> {
        Next::do_sync(CallFn)
    }
}

#[async_trait]
impl<T: Goal> Mission for SyncFn<T> {
    type Goal = T;

    async fn deliver(self, _ctx: &mut Context<Self>) -> Option<Self::Goal> {
        self.output
    }
}

struct CallFn;

impl<T: Goal> DoSync<CallFn> for SyncFn<T> {
    fn once(&mut self, _state: &mut CallFn) -> Result<Next<Self>> {
        let func = self.func.take().unwrap();
        let output = func();
        self.output = Some(output);
        Ok(Next::done())
    }
}
