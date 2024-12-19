use crate::Conductor;
use async_trait::async_trait;
use crb_actor::runtime::ActorRuntime;
use crb_actor::Actor;
use crb_runtime::{Context, Failures, Interruptor, Runtime};

pub trait ConductedActor: Actor {
    type Input: Send;
    type Output;

    fn input(input: Self::Input) -> Self;
    fn output(self) -> Self::Output;
}

pub struct ConductedActorRuntime<C: Conductor, A: ConductedActor> {
    conductor: <C::Context as Context>::Address,
    input: A::Input,
}

#[async_trait]
impl<C, A> Runtime for ConductedActorRuntime<C, A>
where
    C: Conductor,
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

    async fn routine(mut self) -> Failures {
        let actor = A::input(self.input);
        let mut runtime = ActorRuntime::new(actor);
        runtime.perform().await;
        let output = runtime.actor.output();
        // TODO: Send the output
        runtime.failures
    }
}
