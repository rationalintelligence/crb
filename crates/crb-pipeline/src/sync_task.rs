use crate::meta::Metadata;
use crate::pipeline::{Pipeline, RoutePoint, RuntimeGenerator, StageReport};
use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use async_trait::async_trait;
use crb_actor::kit::Address;
use crb_runtime::kit::{Interruptor, Runtime};
use crb_task::kit::{SyncTask, SyncTaskRuntime};
use std::marker::PhantomData;

pub mod stage {
    pub use super::*;

    pub type SyncTask<T> = SyncTaskStage<T>;

    pub fn sync_task<T>() -> SyncTaskStage<T> {
        SyncTaskStage::<T>::default()
    }
}

pub struct SyncTaskStage<T> {
    _type: PhantomData<T>,
}

impl<T> Default for SyncTaskStage<T> {
    fn default() -> Self {
        Self { _type: PhantomData }
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
        let generator = SyncTaskStageRuntimeGenerator::<T>::new::<T::Input>();
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

pub struct SyncTaskStageRuntimeGenerator<T> {
    _type: PhantomData<T>,
}

impl<T> SyncTaskStageRuntimeGenerator<T>
where
    T: SyncTask + Stage,
{
    pub fn new<M>() -> impl RuntimeGenerator<Input = M>
    where
        T: Stage<Input = M>,
    {
        Self { _type: PhantomData }
    }
}

unsafe impl<T> Sync for SyncTaskStageRuntimeGenerator<T> {}

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
        let actor = T::from_input(input);
        let runtime = SyncTaskRuntime::new(actor);
        let conducted_runtime = SyncTaskStageRuntime::<T> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
