use crate::meta::Metadata;
use crate::pipeline::{Pipeline, RoutePoint, RuntimeGenerator, StageReport};
use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use async_trait::async_trait;
use crb_actor::kit::Address;
use crb_runtime::kit::{Interruptor, Runtime};
use crb_task::kit::{HybrydTask, HybrydTaskRuntime};

pub mod stage {
    pub use super::*;

    pub type HybrydTask<T> = HybrydTaskStage<T>;

    pub fn hybryd_task<T>() -> HybrydTaskStage<T>
    where
        T: Stage,
        T::Config: Default,
    {
        HybrydTaskStage::<T>::default()
    }
}

pub struct HybrydTaskStage<T: Stage> {
    config: T::Config,
}

impl<T> Default for HybrydTaskStage<T>
where
    T: Stage,
    T::Config: Default,
{
    fn default() -> Self {
        Self {
            config: T::Config::default(),
        }
    }
}

impl<T> StageSource for HybrydTaskStage<T>
where
    T: Stage,
{
    type Stage = T;
    type Key = StageKey<T>;

    fn source(&self) -> Self::Key {
        StageKey::<T>::new()
    }
}

impl<T> StageDestination for HybrydTaskStage<T>
where
    T: HybrydTask + Stage,
{
    type Stage = T;

    fn destination(&self) -> RoutePoint<T::Input, T::State> {
        let generator = HybrydTaskStageRuntimeGenerator::<T>::new(self.config.clone());
        RoutePoint::new(generator)
    }
}

pub struct HybrydTaskStageRuntime<T: HybrydTask + Stage> {
    meta: Metadata,
    pipeline: Address<Pipeline<T::State>>,
    runtime: HybrydTaskRuntime<T>,
}

#[async_trait]
impl<T> Runtime for HybrydTaskStageRuntime<T>
where
    T: HybrydTask + Stage,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.runtime.get_interruptor()
    }

    async fn routine(&mut self) {
        self.runtime.routine().await;
        if let Some(task) = self.runtime.task.as_mut() {
            while let Some(message) = task.next_output().await {
                let msg = StageReport::<T>::new(self.meta, message);
                let res = self.pipeline.send(msg);
                self.runtime.failures.put(res);
            }
        } else {
            // TODO: Report about the error
        }
    }
}

pub struct HybrydTaskStageRuntimeGenerator<T: Stage> {
    config: T::Config,
}

impl<T> HybrydTaskStageRuntimeGenerator<T>
where
    T: HybrydTask + Stage,
{
    pub fn new(config: T::Config) -> impl RuntimeGenerator<Input = T::Input, State = T::State>
    where
        T: Stage,
    {
        Self { config }
    }
}

unsafe impl<T: Stage> Sync for HybrydTaskStageRuntimeGenerator<T> {}

impl<T> RuntimeGenerator for HybrydTaskStageRuntimeGenerator<T>
where
    T: HybrydTask + Stage,
{
    type State = T::State;
    type Input = T::Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline<Self::State>>,
        input: Self::Input,
        state: &mut Self::State,
    ) -> Box<dyn Runtime> {
        let config = self.config.clone();
        let instance = T::construct(config, input, state);
        let runtime = HybrydTaskRuntime::new(instance);
        let conducted_runtime = HybrydTaskStageRuntime::<T> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
