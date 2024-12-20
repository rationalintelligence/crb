use crate::{MessageToRoute, Pipeline, RuntimeGenerator};
use async_trait::async_trait;
use crb_actor::runtime::ActorRuntime;
use crb_actor::{Actor, Address};
use crb_runtime::{Context, Interruptor, Runtime};
use crb_supervisor::ClosedRuntime;
use std::marker::PhantomData;

pub struct ActorRuntimeGenerator<C> {
    _type: PhantomData<C>,
}

impl<C> RuntimeGenerator for ActorRuntimeGenerator<C>
where
    C: ConductedActor,
{
    type Input = C::Input;

    fn generate(&self, input: Self::Input) -> Box<dyn ClosedRuntime> {
        todo!()
    }
}

pub trait ConductedActor: Actor {
    type Input: Send;
    type Output: Sync + Send + Clone;

    fn input(input: Self::Input) -> Self;
    fn output(self) -> Self::Output;
}

pub struct ConductedActorRuntime<A: ConductedActor> {
    conductor: Address<Pipeline>,
    input: Option<A::Input>,
}

#[async_trait]
impl<A> Runtime for ConductedActorRuntime<A>
where
    A: ConductedActor,
    A::Context: Default,
{
    type Context = A::Context;

    fn get_interruptor(&mut self) -> Interruptor {
        // self.runtime.get_interruptor()
        todo!()
    }

    fn address(&self) -> <Self::Context as Context>::Address {
        todo!()
    }

    async fn routine(&mut self) {
        let input = self.input.take().unwrap();
        let actor = A::input(input);
        let mut runtime = ActorRuntime::new(actor);
        Runtime::routine(&mut runtime).await;
        let message = runtime.actor.output();
        let msg = MessageToRoute { message };
        self.conductor.send(msg);
        // TODO: Send the output
    }
}
