use crate::{MessageToRoute, Metadata, Pipeline, RuntimeGenerator};
use async_trait::async_trait;
use crb_actor::runtime::ActorRuntime;
use crb_actor::{Actor, Address};
use crb_runtime::kit::{Interruptor, Runtime};
use std::marker::PhantomData;

// TODO: Implement
// - Metadata for all messages
// - Epochs (metadata)
// - Split (meta)
// - route_map
// - route_split
// - route_merge | works with `(Option<T1>, ...)` tuple or `Vec<T>`

// TODO: Replace with `Stage`: flexible `From` and `Into` pair
pub trait ConductedActor: Actor<Context: Default> {
    type Input: Send;
    type Output: Clone + Sync + Send;

    fn input(input: Self::Input) -> Self;
    fn output(&mut self) -> Self::Output;
}

pub struct ActorRuntimeGenerator<A> {
    _type: PhantomData<A>,
}

impl<A> ActorRuntimeGenerator<A>
where
    A: ConductedActor,
{
    pub fn new<M>() -> impl RuntimeGenerator<Input = M>
    where
        A: ConductedActor<Input = M>,
    {
        Self { _type: PhantomData }
    }
}

unsafe impl<A> Sync for ActorRuntimeGenerator<A> {}

impl<A> RuntimeGenerator for ActorRuntimeGenerator<A>
where
    A: ConductedActor,
{
    type Input = A::Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline>,
        input: Self::Input,
    ) -> Box<dyn Runtime> {
        let actor = A::input(input);
        let runtime = ActorRuntime::new(actor);
        let conducted_runtime = ConductedActorRuntime::<A> {
            meta,
            pipeline,
            runtime,
        };
        Box::new(conducted_runtime)
    }
}

pub struct ConductedActorRuntime<A: ConductedActor> {
    meta: Metadata,
    pipeline: Address<Pipeline>,
    runtime: ActorRuntime<A>,
}

#[async_trait]
impl<A> Runtime for ConductedActorRuntime<A>
where
    A: ConductedActor,
    A::Context: Default,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.runtime.get_interruptor()
    }

    async fn routine(&mut self) {
        self.runtime.routine().await;
        let message = self.runtime.actor.output();
        let msg = MessageToRoute::<A> {
            meta: self.meta,
            message,
        };
        let res = self.pipeline.send(msg);
        self.runtime.failures.put(res);
    }
}
