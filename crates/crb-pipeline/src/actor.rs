use crate::meta::Metadata;
use crate::pipeline::{Pipeline, RoutePoint, RuntimeGenerator, StageReport};
use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use async_trait::async_trait;
use crb_actor::kit::{Actor, Address};
use crb_actor::runtime::DoActor;
use crb_runtime::kit::{Interruptor, Runtime};

pub mod stage {
    use super::*;

    pub type Actor<A> = ActorStage<A>;

    pub fn actor<A>() -> ActorStage<A>
    where
        A: Stage,
        A::Config: Default,
    {
        ActorStage::<A>::default()
    }
}

pub struct ActorStage<A: Stage> {
    config: A::Config,
}

impl<A> Default for ActorStage<A>
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

impl<A> StageSource for ActorStage<A>
where
    A: Stage,
{
    type Stage = A;
    type Key = StageKey<A>;

    fn source(&self) -> Self::Key {
        StageKey::<A>::new()
    }
}

impl<A> StageDestination for ActorStage<A>
where
    A: Actor + Stage,
    A::Context: Default,
{
    type Stage = A;

    fn destination(&self) -> RoutePoint<A::Input, A::State> {
        let generator = ActorStageRuntimeGenerator::<A>::new(self.config.clone());
        RoutePoint::new(generator)
    }
}

pub struct ActorStageRuntime<A: Actor + Stage> {
    meta: Metadata,
    pipeline: Address<Pipeline<A::State>>,
    runtime: DoActor<A>,
}

#[async_trait]
impl<A> Runtime for ActorStageRuntime<A>
where
    A: Actor + Stage,
    A::Context: Default,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.runtime.get_interruptor()
    }

    async fn routine(&mut self) {
        self.runtime.routine().await;
        while let Some(message) = self.runtime.actor.next_output().await {
            let msg = StageReport::<A>::new(self.meta, message);
            let res = self.pipeline.send(msg);
            self.runtime.failures.put(res);
        }
    }
}

pub struct ActorStageRuntimeGenerator<A: Stage> {
    config: A::Config,
}

impl<A> ActorStageRuntimeGenerator<A>
where
    A: Actor + Stage,
{
    pub fn new(config: A::Config) -> impl RuntimeGenerator<Input = A::Input, State = A::State>
    where
        A: Stage,
        A::Context: Default,
    {
        Self { config }
    }
}

unsafe impl<A: Stage> Sync for ActorStageRuntimeGenerator<A> {}

impl<A> RuntimeGenerator for ActorStageRuntimeGenerator<A>
where
    A: Actor + Stage,
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
        let runtime = DoActor::new(instance);
        let conducted_runtime = ActorStageRuntime::<A> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
