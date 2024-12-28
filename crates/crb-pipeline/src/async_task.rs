use crate::meta::Metadata;
use crate::pipeline::{Pipeline, RoutePoint, RuntimeGenerator, StageReport};
use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use async_trait::async_trait;
use crb_actor::kit::Address;
use crb_runtime::kit::{Interruptor, Runtime};
use crb_task::kit::{AsyncTask, DoAsync};

pub mod stage {
    use super::*;

    pub type AsyncTask<T> = AsyncTaskStage<T>;

    pub fn task<T>() -> AsyncTaskStage<T>
    where
        T: Stage,
        T::Config: Default,
    {
        AsyncTaskStage::<T>::default()
    }
}

pub struct AsyncTaskStage<T: Stage> {
    config: T::Config,
}

impl<T> Default for AsyncTaskStage<T>
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

impl<T> StageSource for AsyncTaskStage<T>
where
    T: Stage,
{
    type Stage = T;
    type Key = StageKey<T>;

    fn source(&self) -> Self::Key {
        StageKey::<T>::new()
    }
}

impl<T> StageDestination for AsyncTaskStage<T>
where
    T: AsyncTask + Stage,
{
    type Stage = T;

    fn destination(&self) -> RoutePoint<T::Input, T::State> {
        let generator = AsyncTaskStageRuntimeGenerator::<T>::new(self.config.clone());
        RoutePoint::new(generator)
    }
}

pub struct AsyncTaskStageRuntime<T: AsyncTask + Stage> {
    meta: Metadata,
    pipeline: Address<Pipeline<T::State>>,
    runtime: DoAsync<T>,
}

#[async_trait]
impl<T> Runtime for AsyncTaskStageRuntime<T>
where
    T: AsyncTask + Stage,
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

pub struct AsyncTaskStageRuntimeGenerator<T: Stage> {
    config: T::Config,
}

impl<T> AsyncTaskStageRuntimeGenerator<T>
where
    T: AsyncTask + Stage,
{
    pub fn new(config: T::Config) -> impl RuntimeGenerator<Input = T::Input, State = T::State>
    where
        T: Stage,
    {
        Self { config }
    }
}

unsafe impl<T: Stage> Sync for AsyncTaskStageRuntimeGenerator<T> {}

impl<T> RuntimeGenerator for AsyncTaskStageRuntimeGenerator<T>
where
    T: AsyncTask + Stage,
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
        let runtime = DoAsync::new(instance);
        let conducted_runtime = AsyncTaskStageRuntime::<T> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
