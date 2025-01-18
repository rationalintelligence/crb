use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, Context, RunAgent};
use crb_runtime::{
    InteractiveRuntime, InteractiveTask, Interruptor, ReachableContext, Runtime, Task,
};

pub trait Mission: Agent {
    type Goal;

    fn deliver(self, ctx: &mut Context<Self>) -> Option<Self::Goal>;
}

pub trait Observer<M: Mission>: Send {
    fn check(&mut self, goal: &M::Goal) -> Result<()>;
}

pub struct RunMission<M: Mission> {
    pub runtime: RunAgent<M>,
    pub observers: Vec<Box<dyn Observer<M>>>,
}

impl<M: Mission> RunMission<M> {
    pub async fn perform(&mut self) -> Result<Option<M::Goal>> {
        self.runtime.perform().await?;
        if let Some(agent) = self.runtime.agent.take() {
            let output = agent.deliver(&mut self.runtime.context);
            if let Some(output) = output.as_ref() {
                for observer in &mut self.observers {
                    let res = observer.check(output);
                    self.runtime.failures.put(res);
                }
            }
            Ok(output)
        } else {
            Ok(None)
        }
    }
}

impl<M: Mission> Task<M> for RunMission<M> {}
impl<M: Mission> InteractiveTask<M> for RunMission<M> {}

#[async_trait]
impl<T> Runtime for RunMission<T>
where
    T: Mission,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.runtime.get_interruptor()
    }

    async fn routine(&mut self) {
        let result = self.perform().await.map(drop);
        self.runtime.failures.put(result.map(drop));
    }
}

#[async_trait]
impl<M: Mission> InteractiveRuntime for RunMission<M> {
    type Context = M::Context;

    fn address(&self) -> <Self::Context as ReachableContext>::Address {
        self.runtime.address()
    }
}
