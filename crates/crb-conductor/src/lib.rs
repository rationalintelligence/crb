/*
pub mod actor;

pub use actor::ConductedActor;
*/

use async_trait::async_trait;
use crb_actor::{Actor, ActorSession};
use crb_supervisor::{ClosedRuntime, Supervisor, SupervisorSession};
use std::any::type_name;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use typedmap::{TypedDashMap, TypedMapKey};

pub struct Conductor {
    routes: TypedDashMap,
}

impl Supervisor for Conductor {
    type GroupBy = ();
}

impl Actor for Conductor {
    type Context = SupervisorSession<Self>;
}

#[async_trait]
trait OnInput<M>: Actor {
    fn on_input(&mut self, message: M, ctx: &mut Self::Context);
}

struct OnInputMessage<M> {
    message: M,
}

impl<M> Hash for MessageRoute<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        type_name::<M>().hash(state);
    }
}

impl<M> PartialEq for MessageRoute<M> {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl<M> Eq for MessageRoute<M> {}

struct MessageRoute<M> {
    _type: PhantomData<M>,
}

impl<M> MessageRoute<M> {
    fn this() -> Self {
        Self { _type: PhantomData }
    }
}

impl<M: 'static> TypedMapKey for MessageRoute<M> {
    type Value = Vec<Box<dyn OnInputRuntimeGenerator<Input = M>>>;
}

trait OnInputRuntimeGenerator: Send + Sync {
    type Input;

    fn generate(&self, input: Self::Input) -> Box<dyn ClosedRuntime>;
}

#[async_trait]
impl<M> OnInput<M> for Conductor
where
    M: Send + Sync + Clone + 'static,
{
    fn on_input(&mut self, message: M, ctx: &mut Self::Context) {
        let generators = self.routes.get(&MessageRoute::<M>::this());
        if let Some(generators) = generators {
            for generator in generators.iter() {
                let runtime = generator.generate(message.clone());
                ctx.spawn_trackable(runtime, ());
            }
        }
        // TODO: Use the routing table to forward a message
    }
}
