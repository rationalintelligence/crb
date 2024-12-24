use crate::meta::Metadata;
use crate::pipeline::{Pipeline, RoutePoint, RuntimeGenerator, StageReport};
use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use async_trait::async_trait;
use crb_actor::kit::Address;
use crb_runtime::kit::{Interruptor, Runtime};
use crb_task::kit::{Task, TaskRuntime};

pub mod stage {
    use super::*;

    pub type Task<T> = TaskStage<T>;

    pub fn task<T>() -> TaskStage<T>
    where
        T: Stage,
        T::Config: Default,
    {
        TaskStage::<T>::default()
    }
}

pub struct TaskStage<T: Stage> {
    config: T::Config,
}

impl<T> Default for TaskStage<T>
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
        let generator = TaskStageRuntimeGenerator::<T>::new::<T::Input>(self.config.clone());
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

pub struct TaskStageRuntimeGenerator<T: Stage> {
    config: T::Config,
}

impl<T> TaskStageRuntimeGenerator<T>
where
    T: Task + Stage,
{
    pub fn new<M>(config: T::Config) -> impl RuntimeGenerator<Input = M>
    where
        T: Stage<Input = M>,
    {
        Self { config }
    }
}

unsafe impl<T: Stage> Sync for TaskStageRuntimeGenerator<T> {}

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
        let config = self.config.clone();
        let instance = T::construct(config, input);
        let runtime = TaskRuntime::new(instance);
        let conducted_runtime = TaskStageRuntime::<T> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
