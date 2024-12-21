use crate::{MessageToRoute, Pipeline, RuntimeGenerator};
use async_trait::async_trait;
use crb_actor::runtime::ActorRuntime;
use crb_actor::{Actor, Address};
use crb_runtime::{Interruptor, Runtime};
use std::marker::PhantomData;

pub struct ActorRuntimeGenerator<C> {
    _type: PhantomData<C>,
}

unsafe impl<A> Sync for ActorRuntimeGenerator<A> {}

impl<A> RuntimeGenerator for ActorRuntimeGenerator<A>
where
    A: ConductedActor,
{
    type Input = A::Input;

    fn generate(&self, pipeline: Address<Pipeline>, input: Self::Input) -> Box<dyn Runtime> {
        let runtime = ConductedActorRuntime::<A> {
            pipeline,
            input: Some(input),
        };
        Box::new(runtime)
    }
}

pub trait ConductedActor: Actor<Context: Default> {
    type Input: Send;
    type Output: Sync + Send + Clone;

    fn input(input: Self::Input) -> Self;
    fn output(self) -> Self::Output;
}

pub struct ConductedActorRuntime<A: ConductedActor> {
    pipeline: Address<Pipeline>,
    input: Option<A::Input>,
}

#[async_trait]
impl<A> Runtime for ConductedActorRuntime<A>
where
    A: ConductedActor,
    A::Context: Default,
{
    fn get_interruptor(&mut self) -> Interruptor {
        // self.runtime.get_interruptor()
        todo!()
    }

    async fn routine(&mut self) {
        let input = self.input.take().unwrap();
        let actor = A::input(input);
        let mut runtime = ActorRuntime::new(actor);
        Runtime::routine(&mut runtime).await;
        let message = runtime.actor.output();
        let msg = MessageToRoute { message };
        self.pipeline.send(msg);
    }
}
