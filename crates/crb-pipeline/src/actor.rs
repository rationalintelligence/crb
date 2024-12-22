use crate::stage::{Stage, StageDestination, StageKey, StageReport, StageSource};
use crate::{Metadata, Pipeline, RoutePoint, RuntimeGenerator};
use async_trait::async_trait;
use crb_actor::kit::{Actor, Address};
use crb_actor::runtime::ActorRuntime;
use crb_runtime::kit::{Interruptor, Runtime};
use std::marker::PhantomData;

pub struct ActorStage<A> {
    _type: PhantomData<A>,
}

impl<A> Default for ActorStage<A> {
    fn default() -> Self {
        Self { _type: PhantomData }
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

    fn destination(&self) -> RoutePoint<A::Input> {
        let generator = ActorStageRuntimeGenerator::<A>::new::<A::Input>();
        Box::new(generator)
    }
}

pub struct ActorStageRuntime<A: Actor + Stage> {
    meta: Metadata,
    pipeline: Address<Pipeline>,
    runtime: ActorRuntime<A>,
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
        let message = self.runtime.actor.to_output();
        let msg = StageReport::<A>::new(self.meta, message);
        let res = self.pipeline.send(msg);
        self.runtime.failures.put(res);
    }
}

pub struct ActorStageRuntimeGenerator<A> {
    _type: PhantomData<A>,
}

impl<A> ActorStageRuntimeGenerator<A>
where
    A: Actor + Stage,
{
    pub fn new<M>() -> impl RuntimeGenerator<Input = M>
    where
        A: Stage<Input = M>,
        A::Context: Default,
    {
        Self { _type: PhantomData }
    }
}

unsafe impl<A> Sync for ActorStageRuntimeGenerator<A> {}

impl<A> RuntimeGenerator for ActorStageRuntimeGenerator<A>
where
    A: Actor + Stage,
    A::Context: Default,
{
    type Input = A::Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline>,
        input: Self::Input,
    ) -> Box<dyn Runtime> {
        let actor = A::from_input(input);
        let runtime = ActorRuntime::new(actor);
        let conducted_runtime = ActorStageRuntime::<A> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}
