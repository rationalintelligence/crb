use crate::meta::Metadata;
use crate::pipeline::{Pipeline, RoutePoint, RuntimeGenerator, StageReport};
use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use async_trait::async_trait;
use crb_actor::kit::Address;
use crb_routine::kit::{Routine, RoutineRuntime};
use crb_runtime::kit::{Interruptor, Runtime};

pub mod stage {
    use super::*;

    pub type Routine<R> = RoutineStage<R>;

    pub fn routine<R>() -> RoutineStage<R>
    where
        R: Stage,
        R::Config: Default,
    {
        RoutineStage::<R>::default()
    }
}

pub struct RoutineStage<R: Stage> {
    config: R::Config,
}

impl<R> Default for RoutineStage<R>
where
    R: Stage,
    R::Config: Default,
{
    fn default() -> Self {
        Self {
            config: R::Config::default(),
        }
    }
}

impl<R> StageSource for RoutineStage<R>
where
    R: Stage,
{
    type Stage = R;
    type Key = StageKey<R>;

    fn source(&self) -> Self::Key {
        StageKey::<R>::new()
    }
}

impl<R> StageDestination for RoutineStage<R>
where
    R: Routine + Stage,
    R::Context: Default,
{
    type Stage = R;

    fn destination(&self) -> RoutePoint<R::Input, R::State> {
        let generator = RoutineStageRuntimeGenerator::<R>::new(self.config.clone());
        RoutePoint::new(generator)
    }
}

pub struct RoutineStageRuntime<R: Routine + Stage> {
    meta: Metadata,
    pipeline: Address<Pipeline<R::State>>,
    runtime: RoutineRuntime<R>,
}

#[async_trait]
impl<R> Runtime for RoutineStageRuntime<R>
where
    R: Routine + Stage,
    R::Context: Default,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.runtime.get_interruptor()
    }

    async fn routine(&mut self) {
        self.runtime.routine().await;
        while let Some(message) = self.runtime.routine.next_output().await {
            let msg = StageReport::<R>::new(self.meta, message);
            let res = self.pipeline.send(msg);
            self.runtime.failures.put(res);
        }
    }
}

pub struct RoutineStageRuntimeGenerator<R: Stage> {
    config: R::Config,
}

impl<R> RoutineStageRuntimeGenerator<R>
where
    R: Routine + Stage,
{
    pub fn new(config: R::Config) -> impl RuntimeGenerator<Input = R::Input, State = R::State>
    where
        R: Stage,
        R::Context: Default,
    {
        Self { config }
    }
}

unsafe impl<R: Stage> Sync for RoutineStageRuntimeGenerator<R> {}

impl<R> RuntimeGenerator for RoutineStageRuntimeGenerator<R>
where
    R: Routine + Stage,
    R::Context: Default,
{
    type State = R::State;
    type Input = R::Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline<Self::State>>,
        input: Self::Input,
        state: &mut Self::State,
    ) -> Box<dyn Runtime> {
        let config = self.config.clone();
        let instance = R::construct(config, input, state);
        let runtime = RoutineRuntime::new(instance);
        let conducted_runtime = RoutineStageRuntime::<R> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
