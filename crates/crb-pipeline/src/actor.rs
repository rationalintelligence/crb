use crate::{MessageToRoute, Pipeline, RuntimeGenerator};
use async_trait::async_trait;
use crb_actor::runtime::ActorRuntime;
use crb_actor::{Actor, Address};
use crb_runtime::{Interruptor, Runtime};
use std::marker::PhantomData;

pub struct ActorRuntimeGenerator<A> {
    _type: PhantomData<A>,
}

impl<A> ActorRuntimeGenerator<A>
where
    A: ConductedActor,
{
    pub fn new<FROM>() -> impl RuntimeGenerator<Input = FROM::Output>
    where
        FROM: ConductedActor,
        A: ConductedActor<Input = FROM::Output>,
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

    fn generate(&self, pipeline: Address<Pipeline>, input: Self::Input) -> Box<dyn Runtime> {
        let actor = A::input(input);
        let runtime = ActorRuntime::new(actor);
        let conducted_runtime = ConductedActorRuntime::<A> { pipeline, runtime };
        Box::new(conducted_runtime)
    }
}

pub trait ConductedActor: Actor<Context: Default> {
    type Input: Send;
    type Output: Sync + Send + Clone;

    fn input(input: Self::Input) -> Self;
    fn output(&mut self) -> Self::Output;
}

pub struct ConductedActorRuntime<A: ConductedActor> {
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
        let msg = MessageToRoute::<A> { message };
        self.pipeline.send(msg);
    }
}
