use super::{runtime::RunMission, Goal, Mission};
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, Context, DoAsync, Next};
use futures::Future;

impl<T: Goal> RunMission<AsyncFn<T>> {
    pub fn new_async<F: AnyAsyncFut<T>>(fut: F) -> Self {
        let task = AsyncFn::<T> {
            fut: Some(Box::new(fut)),
            output: None,
        };
        Self::new(task)
    }
}

pub trait AnyAsyncFut<T>: Future<Output = T> + Send + 'static {}

impl<F, T> AnyAsyncFut<T> for F where F: Future<Output = T> + Send + 'static {}

pub struct AsyncFn<T> {
    fut: Option<Box<dyn AnyAsyncFut<T>>>,
    output: Option<T>,
}

impl<T: Goal> Agent for AsyncFn<T> {
    type Context = AgentSession<Self>;

    fn initialize(&mut self, _ctx: &mut Context<Self>) -> Next<Self> {
        Next::do_async(CallFn)
    }
}

#[async_trait]
impl<T: Goal> Mission for AsyncFn<T> {
    type Goal = T;

    async fn deliver(self, _ctx: &mut Context<Self>) -> Option<Self::Goal> {
        self.output
    }
}

struct CallFn;

#[async_trait]
impl<T: Goal> DoAsync<CallFn> for AsyncFn<T> {
    async fn once(&mut self, _state: &mut CallFn) -> Result<Next<Self>> {
        let fut = self.fut.take().unwrap();
        let pinned_fut = Box::into_pin(fut);
        let output = pinned_fut.await;
        self.output = Some(output);
        Ok(Next::done())
    }
}
