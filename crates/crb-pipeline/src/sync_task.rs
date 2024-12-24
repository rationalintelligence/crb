use crate::meta::Metadata;
use crate::pipeline::{Pipeline, RoutePoint, RuntimeGenerator, StageReport};
use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use async_trait::async_trait;
use crb_actor::kit::Address;
use crb_runtime::kit::{Interruptor, Runtime};
use crb_task::kit::{SyncTask, SyncTaskRuntime};

pub mod stage {
    pub use super::*;

    pub type SyncTask<T> = SyncTaskStage<T>;

    pub fn sync_task<T>() -> SyncTaskStage<T>
    where
        T: Stage,
        T::Config: Default,
    {
        SyncTaskStage::<T>::default()
    }
}

pub struct SyncTaskStage<T: Stage> {
    config: T::Config,
}

impl<T> Default for SyncTaskStage<T>
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

impl<T> StageSource for SyncTaskStage<T>
where
    T: Stage,
{
    type Stage = T;
    type Key = StageKey<T>;

    fn source(&self) -> Self::Key {
        StageKey::<T>::new()
    }
}

impl<T> StageDestination for SyncTaskStage<T>
where
    T: SyncTask + Stage,
{
    type Stage = T;

    fn destination(&self) -> RoutePoint<T::Input> {
        let generator = SyncTaskStageRuntimeGenerator::<T>::new::<T::Input>(self.config.clone());
        Box::new(generator)
    }
}

pub struct SyncTaskStageRuntime<T: SyncTask + Stage> {
    meta: Metadata,
    pipeline: Address<Pipeline>,
    runtime: SyncTaskRuntime<T>,
}

#[async_trait]
impl<T> Runtime for SyncTaskStageRuntime<T>
where
    T: SyncTask + Stage,
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

pub struct SyncTaskStageRuntimeGenerator<T: Stage> {
    config: T::Config,
}

impl<T> SyncTaskStageRuntimeGenerator<T>
where
    T: SyncTask + Stage,
{
    pub fn new<M>(config: T::Config) -> impl RuntimeGenerator<Input = M>
    where
        T: Stage<Input = M>,
    {
        Self { config }
    }
}

unsafe impl<T: Stage> Sync for SyncTaskStageRuntimeGenerator<T> {}

impl<T> RuntimeGenerator for SyncTaskStageRuntimeGenerator<T>
where
    T: SyncTask + Stage,
{
    type Input = T::Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline>,
        input: Self::Input,
    ) -> Box<dyn Runtime> {
        let config = self.config.clone();
        let instance = T::construct(config, input);
        let runtime = SyncTaskRuntime::new(instance);
        let conducted_runtime = SyncTaskStageRuntime::<T> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
