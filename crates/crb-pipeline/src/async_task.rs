use crate::meta::Metadata;
use crate::pipeline::{Pipeline, RoutePoint, RuntimeGenerator, StageReport};
use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use async_trait::async_trait;
use crb_actor::kit::Address;
use crb_runtime::kit::{Interruptor, Runtime};
use crb_task::kit::{Task, TaskRuntime};
use std::marker::PhantomData;

pub mod stage {
    use super::*;

    pub type Task<T> = TaskStage<T>;

    pub fn task<T>() -> TaskStage<T> {
        TaskStage::<T>::default()
    }
}

pub struct TaskStage<T> {
    _type: PhantomData<T>,
}

impl<T> Default for TaskStage<T> {
    fn default() -> Self {
        Self { _type: PhantomData }
    }
}

impl<T> StageSource for TaskStage<T>
where
    T: Stage,
{
    type Stage = T;
    type Key = StageKey<T>;

    fn source(&self) -> Self::Key {
        StageKey::<T>::new()
    }
}

impl<T> StageDestination for TaskStage<T>
where
    T: Task + Stage,
{
    type Stage = T;

    fn destination(&self) -> RoutePoint<T::Input> {
        let generator = TaskStageRuntimeGenerator::<T>::new::<T::Input>();
        Box::new(generator)
    }
}

pub struct TaskStageRuntime<T: Task + Stage> {
    meta: Metadata,
    pipeline: Address<Pipeline>,
    runtime: TaskRuntime<T>,
}

#[async_trait]
impl<T> Runtime for TaskStageRuntime<T>
where
    T: Task + Stage,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.runtime.get_interruptor()
    }

    async fn routine(&mut self) {
        self.runtime.routine().await;
        while let Some(message) = self.runtime.task.next_output().await {
            let msg = StageReport::<T>::new(self.meta, message);
            let res = self.pipeline.send(msg);
            self.runtime.failures.put(res);
        }
    }
}

pub struct TaskStageRuntimeGenerator<T> {
    _type: PhantomData<T>,
}

impl<T> TaskStageRuntimeGenerator<T>
where
    T: Task + Stage,
{
    pub fn new<M>() -> impl RuntimeGenerator<Input = M>
    where
        T: Stage<Input = M>,
    {
        Self { _type: PhantomData }
    }
}

unsafe impl<T> Sync for TaskStageRuntimeGenerator<T> {}

impl<T> RuntimeGenerator for TaskStageRuntimeGenerator<T>
where
    T: Task + Stage,
{
    type Input = T::Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline>,
        input: Self::Input,
    ) -> Box<dyn Runtime> {
        let actor = T::from_input(input);
        let runtime = TaskRuntime::new(actor);
        let conducted_runtime = TaskStageRuntime::<T> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
