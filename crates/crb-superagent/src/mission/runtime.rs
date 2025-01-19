use super::{Mission, Observer};
use async_trait::async_trait;
use crb_agent::RunAgent;
use crb_runtime::{
    InteractiveRuntime, InteractiveTask, Interruptor, ReachableContext, Runtime, Task,
};

pub struct RunMission<M: Mission> {
    pub runtime: RunAgent<M>,
    pub observers: Vec<Box<dyn Observer<M>>>,
}

impl<M: Mission> RunMission<M> {
    pub fn new(mission: M) -> Self
    where
        M::Context: Default,
    {
        Self {
            runtime: RunAgent::new(mission),
            observers: Vec::new(),
        }
    }

    pub async fn perform(&mut self) -> Option<M::Goal> {
        self.runtime.perform().await;
        if let Some(agent) = self.runtime.agent.take() {
            let output = agent.deliver(&mut self.runtime.context).await;
            if let Some(output) = output.as_ref() {
                for observer in &mut self.observers {
                    observer.check(output).ok();
                }
            }
            let interrupted = output.is_none();
            self.runtime.report(interrupted);
            output
        } else {
            self.runtime.report(true);
            None
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
        self.perform().await;
    }
}

#[async_trait]
impl<M: Mission> InteractiveRuntime for RunMission<M> {
    type Context = M::Context;

    fn address(&self) -> <Self::Context as ReachableContext>::Address {
        self.runtime.address()
    }
}
