pub mod actor;

use actor::ConductedActor;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_actor::{Actor, MessageFor};
use crb_supervisor::{ClosedRuntime, Supervisor, SupervisorSession};
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
        TO: ConductedActor,
    {
        let key = RouteKey::<FROM::Output>::new();
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

trait RuntimeGenerator: Send + Sync {
    type Input;

    fn generate(&self, input: Self::Input) -> Box<dyn ClosedRuntime>;
}

struct MessageToRoute<M> {
    message: M,
}

#[async_trait]
impl<M> MessageFor<Pipeline> for MessageToRoute<M>
where
    M: Clone + Sync + Send + 'static,
{
    async fn handle(
        self: Box<Self>,
        actor: &mut Pipeline,
        ctx: &mut SupervisorSession<Pipeline>,
    ) -> Result<(), Error> {
        let generators = actor.routes.get(&RouteKey::<M>::new());
        if let Some(generators) = generators {
            for generator in generators.iter() {
                let runtime = generator.generate(self.message.clone());
                ctx.spawn_trackable(runtime, ());
            }
        }
        Ok(())
    }
}
