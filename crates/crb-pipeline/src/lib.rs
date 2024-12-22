pub mod actor;
pub mod extension;
pub mod meta;
pub mod routine;
pub mod service;
pub mod stage;

pub mod kit {
    pub use crate::actor::ActorStage;
    pub use crate::extension::AddressExt;
    pub use crate::service::InputStage;
    pub use crate::stage::Stage;
    pub use crate::Pipeline;
}

use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_actor::{Actor, Address, MessageFor};
use crb_core::types::Clony;
use crb_runtime::kit::{Context, Runtime};
use crb_supervisor::{Supervisor, SupervisorSession};
use meta::{Metadata, Sequencer};
use service::InitialKey;
use stage::Stage;
use stage::{StageDestination, StageSource};
use std::any::type_name;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use typedmap::{TypedDashMap, TypedMapKey};

pub struct Pipeline {
    sequencer: Sequencer,
    routes: TypedDashMap,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            sequencer: Sequencer::default(),
            routes: TypedDashMap::default(),
        }
    }

    pub fn stage<FROM, TO>(&mut self, from: FROM, to: TO)
    where
        FROM: StageSource,
        TO: StageDestination,
        TO::Stage: Stage<Input = <FROM::Stage as Stage>::Output>,
    {
        let key = from.source();
        let generator = to.destination();
        self.routes.entry(key).or_default().push(generator);
    }

    /*
    pub fn input<M, TO>(&mut self)
    where
        M: Clony,
        TO: ConductedActor<Input = M>,
    {
        let key = InitialKey::<M>::new();
        let generator = ActorRuntimeGenerator::<TO>::new::<M>();
        let value = Box::new(generator);
        self.routes.entry(key).or_default().push(value);
    }

    pub fn route<FROM, TO>(&mut self)
    where
        FROM: ConductedActor,
        TO: ConductedActor<Input = FROM::Output>,
    {
        let key = RouteKey::<FROM>::new();
        let generator = ActorRuntimeGenerator::<TO>::new::<FROM::Output>();
        let value = Box::new(generator);
        self.routes.entry(key).or_default().push(value);
    }
    */
}

impl Supervisor for Pipeline {
    type GroupBy = ();
}

impl Actor for Pipeline {
    type Context = SupervisorSession<Self>;
}

/*
pub struct RouteKey<A> {
    _type: PhantomData<A>,
}

unsafe impl<A> Sync for RouteKey<A> {}

// TODO: Use `Stage` instead of `A`
impl<A> RouteKey<A> {
    fn new() -> Self {
        Self { _type: PhantomData }
    }
}

impl<A> Clone for RouteKey<A> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<A> Hash for RouteKey<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        type_name::<A>().hash(state);
    }
}

impl<A> PartialEq for RouteKey<A> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<A> Eq for RouteKey<A> {}

impl<A: ConductedActor> TypedMapKey for RouteKey<A> {
    type Value = RouteValue<A::Output>;
}
*/

type RoutePoint<M> = Box<dyn RuntimeGenerator<Input = M>>;
type RouteValue<M> = Vec<RoutePoint<M>>;

pub trait RuntimeGenerator: Send + Sync {
    type Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline>,
        input: Self::Input,
    ) -> Box<dyn Runtime>;
}

// TODO: Move?
struct InitialMessage<M> {
    message: M,
}

impl<M> InitialMessage<M> {
    fn new(message: M) -> Self {
        Self { message }
    }
}

#[async_trait]
impl<M> MessageFor<Pipeline> for InitialMessage<M>
where
    M: Clony,
{
    async fn handle(
        self: Box<Self>,
        actor: &mut Pipeline,
        ctx: &mut SupervisorSession<Pipeline>,
    ) -> Result<(), Error> {
        let layer = actor.sequencer.next();
        let meta = Metadata::new(layer);
        let key = InitialKey::<M>::new();
        actor.spawn_workers(meta, key, self.message, ctx);
        Ok(())
    }
}

/*
struct MessageToRoute<A: ConductedActor> {
    meta: Metadata,
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
        let key = RouteKey::<A>::new();
        actor.spawn_workers(self.meta, key, self.message, ctx);
        Ok(())
    }
}
*/

impl Pipeline {
    fn spawn_workers<K, M>(
        &mut self,
        meta: Metadata,
        key: K,
        message: M,
        ctx: &mut SupervisorSession<Pipeline>,
    ) where
        K: TypedMapKey<Value = RouteValue<M>> + Send + Sync + 'static,
        M: Clone + 'static,
    {
        let generators = self.routes.get(&key);
        if let Some(generators) = generators {
            if generators.is_empty() {
                log::error!("Workers for {} are not presented.", type_name::<M>());
            }
            for generator in generators.iter() {
                let pipeline = ctx.address().clone();
                let message = message.clone();
                let runtime = generator.generate(meta, pipeline, message);
                ctx.spawn_trackable(runtime, ());
            }
        }
    }
}
