use crate::stage::{Stage, StageDestination, StageKey, StageSource};
use crate::{Metadata, Pipeline, RoutePoint, RuntimeGenerator};
use anyhow::Error;
use async_trait::async_trait;
use crb_actor::runtime::ActorRuntime;
use crb_actor::MessageFor;
use crb_actor::{Actor, Address};
use crb_runtime::kit::{Interruptor, Runtime};
use crb_supervisor::SupervisorSession;
use std::marker::PhantomData;

// TODO: Implement
// - Metadata for all messages
// - Epochs (metadata)
// - Split (meta)
// - route_map
// - route_split
// - route_merge | works with `(Option<T1>, ...)` tuple or `Vec<T>`

pub struct ActorStage<A> {
    _type: PhantomData<A>,
}

impl<A> ActorStage<A> {
    pub fn stage() -> Self {
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
        let msg = ActorStageRuntimeReport::<A> {
            meta: self.meta,
            message,
        };
        let res = self.pipeline.send(msg);
        self.runtime.failures.put(res);
    }
}

struct ActorStageRuntimeReport<A: Stage> {
    meta: Metadata,
    message: A::Output,
}

#[async_trait]
impl<A> MessageFor<Pipeline> for ActorStageRuntimeReport<A>
where
    A: Stage,
{
    async fn handle(
        self: Box<Self>,
        actor: &mut Pipeline,
        ctx: &mut SupervisorSession<Pipeline>,
    ) -> Result<(), Error> {
        let key = StageKey::<A>::new();
        actor.spawn_workers(self.meta, key, self.message, ctx);
        Ok(())
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
