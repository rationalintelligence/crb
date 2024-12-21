pub mod actor;

use actor::{ActorRuntimeGenerator, ConductedActor};
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_actor::{Actor, Address, MessageFor};
use crb_runtime::{Context, Runtime};
use crb_supervisor::{Supervisor, SupervisorSession};
use std::any::type_name;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use typedmap::{TypedDashMap, TypedMapKey};

pub struct Pipeline {
    routes: TypedDashMap,
}

impl Pipeline {
    pub fn route<FROM, TO>(&mut self)
    where
        FROM: ConductedActor,
        TO: ConductedActor<Input = FROM::Output>,
    {
        let key = RouteKey::<FROM::Output>::new();
        let generator = ActorRuntimeGenerator::<TO>::new::<FROM>();
        let value = Box::new(generator);
        self.routes.entry(key).or_default().push(value);
    }
}

impl Supervisor for Pipeline {
    type GroupBy = ();
}

impl Actor for Pipeline {
    type Context = SupervisorSession<Self>;
}

impl<M> Hash for RouteKey<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        type_name::<M>().hash(state);
    }
}

impl<M> PartialEq for RouteKey<M> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<M> Eq for RouteKey<M> {}

struct RouteKey<M> {
    _type: PhantomData<M>,
}

impl<M> RouteKey<M> {
    fn new() -> Self {
        Self { _type: PhantomData }
    }
}

impl<M: 'static> TypedMapKey for RouteKey<M> {
    type Value = Vec<Box<dyn RuntimeGenerator<Input = M>>>;
}

pub trait RuntimeGenerator: Send + Sync {
    type Input;

    fn generate(&self, pipeline: Address<Pipeline>, input: Self::Input) -> Box<dyn Runtime>;
}

struct MessageToRoute<A: ConductedActor> {
    message: A::Output,
}

#[async_trait]
impl<A> MessageFor<Pipeline> for MessageToRoute<A>
where
    A: ConductedActor,
{
    async fn handle(
        self: Box<Self>,
        actor: &mut Pipeline,
        ctx: &mut SupervisorSession<Pipeline>,
    ) -> Result<(), Error> {
        let key = RouteKey::<A::Output>::new();
        let generators = actor.routes.get(&key);
        if let Some(generators) = generators {
            for generator in generators.iter() {
                let pipeline = ctx.address().clone();
                let message = self.message.clone();
                let runtime = generator.generate(pipeline, message);
                ctx.spawn_trackable(runtime, ());
            }
        }
        Ok(())
    }
}
