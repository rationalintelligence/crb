use crate::meta::Metadata;
use crate::pipeline::{Pipeline, RoutePoint, RuntimeGenerator, StageReport};
use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use async_trait::async_trait;
use crb_actor::kit::Address;
use crb_routine::kit::{Routine, RoutineRuntime};
use crb_runtime::kit::{Interruptor, Runtime};
use std::marker::PhantomData;

pub mod stage {
    use super::*;

    pub type Routine<R> = RoutineStage<R>;

    pub fn routine<R>() -> RoutineStage<R> {
        RoutineStage::<R>::default()
    }
}

pub struct RoutineStage<R> {
    _type: PhantomData<R>,
}

impl<R> Default for RoutineStage<R> {
    fn default() -> Self {
        Self { _type: PhantomData }
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

    fn destination(&self) -> RoutePoint<R::Input> {
        let generator = RoutineStageRuntimeGenerator::<R>::new::<R::Input>();
        Box::new(generator)
    }
}

pub struct RoutineStageRuntime<R: Routine + Stage> {
    meta: Metadata,
    pipeline: Address<Pipeline>,
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
        let message = self.runtime.routine.to_output();
        let msg = StageReport::<R>::new(self.meta, message);
        let res = self.pipeline.send(msg);
        self.runtime.failures.put(res);
    }
}

pub struct RoutineStageRuntimeGenerator<R> {
    _type: PhantomData<R>,
}

impl<R> RoutineStageRuntimeGenerator<R>
where
    R: Routine + Stage,
{
    pub fn new<M>() -> impl RuntimeGenerator<Input = M>
    where
        R: Stage<Input = M>,
        R::Context: Default,
    {
        Self { _type: PhantomData }
    }
}

unsafe impl<R> Sync for RoutineStageRuntimeGenerator<R> {}

impl<R> RuntimeGenerator for RoutineStageRuntimeGenerator<R>
where
    R: Routine + Stage,
    R::Context: Default,
{
    type Input = R::Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline>,
        input: Self::Input,
    ) -> Box<dyn Runtime> {
        let actor = R::from_input(input);
        let runtime = RoutineRuntime::new(actor);
        let conducted_runtime = RoutineStageRuntime::<R> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
