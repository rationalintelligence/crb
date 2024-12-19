use async_trait::async_trait;
use crb_actor::runtime::ActorRuntime;
use crb_actor::Actor;
use crb_runtime::{Failures, Interruptor, Runtime};

pub trait ConductedActor: Actor {
    type Input: Send;
    type Output;

    fn input(input: Self::Input) -> Self;
    fn output(self) -> Self::Output;
}

pub struct ConductedActorRuntime<A: ConductedActor> {
    input: A::Input,
}

#[async_trait]
impl<A> Runtime for ConductedActorRuntime<A>
where
    A: ConductedActor,
    A::Context: Default,
{
    type Context = A::Context;

    fn get_interruptor(&mut self) -> Box<dyn Interruptor> {
        // self.runtime.get_interruptor()
        todo!()
    }

    async fn routine(mut self) -> Failures {
        let actor = A::input(self.input);
        let mut runtime = ActorRuntime::new(actor);
        runtime.execute().await;
        let output = runtime.actor.output();
        // TODO: Send the output
        runtime.failures
    }

    fn context(&self) -> &Self::Context {
        todo!()
        // self.runtime.context()
    }
}
