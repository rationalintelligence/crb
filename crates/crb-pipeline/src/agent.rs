use crate::meta::Metadata;
use crate::pipeline::{Pipeline, RoutePoint, RuntimeGenerator, StageReport};
use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use async_trait::async_trait;
use crb_agent::{Address, Agent, RunAgent};
use crb_runtime::{Interruptor, Runtime};

pub mod stage {
    use super::*;

    pub type Agent<A> = AgentStage<A>;

    pub fn agent<A>() -> AgentStage<A>
    where
        A: Stage,
        A::Config: Default,
    {
        AgentStage::<A>::default()
    }
}

pub struct AgentStage<A: Stage> {
    config: A::Config,
}

impl<A> Default for AgentStage<A>
where
    A: Stage,
    A::Config: Default,
{
    fn default() -> Self {
        Self {
            config: A::Config::default(),
        }
    }
}

impl<A> StageSource for AgentStage<A>
where
    A: Stage,
{
    type Stage = A;
    type Key = StageKey<A>;

    fn source(&self) -> Self::Key {
        StageKey::<A>::new()
    }
}

impl<A> StageDestination for AgentStage<A>
where
    A: Agent + Stage,
    A::Context: Default,
{
    type Stage = A;

    fn destination(&self) -> RoutePoint<A::Input, A::State> {
        let generator = AgentStageRuntimeGenerator::<A>::generator(self.config.clone());
        RoutePoint::new(generator)
    }
}

pub struct AgentStageRuntime<A: Agent + Stage> {
    meta: Metadata,
    pipeline: Address<Pipeline<A::State>>,
    runtime: RunAgent<A>,
}

#[async_trait]
impl<A> Runtime for AgentStageRuntime<A>
where
    A: Agent + Stage,
    A::Context: Default,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.runtime.get_interruptor()
    }

    async fn routine(&mut self) {
        // TODO: Setup finalizers instead
        self.runtime.routine().await;
        if let Some(agent) = self.runtime.agent.as_mut() {
            while let Some(message) = agent.next_output().await {
                let msg = StageReport::<A>::new(self.meta, message);
                let res = self.pipeline.send(msg);
                self.runtime.failures.put(res);
            }
        }
    }
}

pub struct AgentStageRuntimeGenerator<A: Stage> {
    config: A::Config,
}

impl<A> AgentStageRuntimeGenerator<A>
where
    A: Agent + Stage,
{
    pub fn generator(config: A::Config) -> impl RuntimeGenerator<Input = A::Input, State = A::State>
    where
        A: Stage,
        A::Context: Default,
    {
        Self { config }
    }
}

unsafe impl<A: Stage> Sync for AgentStageRuntimeGenerator<A> {}

impl<A> RuntimeGenerator for AgentStageRuntimeGenerator<A>
where
    A: Agent + Stage,
    A::Context: Default,
{
    type State = A::State;
    type Input = A::Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline<Self::State>>,
        input: Self::Input,
        state: &mut Self::State,
    ) -> Box<dyn Runtime> {
        let config = self.config.clone();
        let instance = A::construct(config, input, state);
        let runtime = RunAgent::new(instance);
        let conducted_runtime = AgentStageRuntime::<A> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
