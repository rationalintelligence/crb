pub mod actor;
pub mod extension;
pub mod sequencer;

pub use actor::ConductedActor;
pub use extension::AddressExt;

use actor::ActorRuntimeGenerator;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_actor::{Actor, Address, MessageFor};
use crb_runtime::{Context, Runtime};
use crb_supervisor::{Supervisor, SupervisorSession};
use sequencer::{SeqId, Sequencer};
use std::any::type_name;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use typedmap::{TypedDashMap, TypedMapKey};

pub trait DistributableMessage: Clone + Sync + Send + 'static {}

impl<M> DistributableMessage for M where Self: Clone + Sync + Send + 'static {}

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

    pub fn input<M, TO>(&mut self)
    where
        M: DistributableMessage,
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
}

impl Supervisor for Pipeline {
    type GroupBy = ();
}

impl Actor for Pipeline {
    type Context = SupervisorSession<Self>;
}

struct RouteKey<A> {
    _type: PhantomData<A>,
}

unsafe impl<A> Sync for RouteKey<A> {}

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

struct InitialKey<M> {
    _type: PhantomData<M>,
}

impl<M> InitialKey<M> {
    fn new() -> Self {
        Self { _type: PhantomData }
    }
}

impl<M> Clone for InitialKey<M> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<M> Hash for InitialKey<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        type_name::<M>().hash(state);
    }
}

impl<M> PartialEq for InitialKey<M> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<M> Eq for InitialKey<M> {}

impl<M: 'static> TypedMapKey for InitialKey<M> {
    type Value = RouteValue<M>;
}

type RouteValue<M> = Vec<Box<dyn RuntimeGenerator<Input = M>>>;

pub trait RuntimeGenerator: Send + Sync {
    type Input;

    fn generate(
        &self,
        seq_id: SeqId,
        pipeline: Address<Pipeline>,
        input: Self::Input,
    ) -> Box<dyn Runtime>;
}

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
    M: DistributableMessage,
{
    async fn handle(
        self: Box<Self>,
        actor: &mut Pipeline,
        ctx: &mut SupervisorSession<Pipeline>,
    ) -> Result<(), Error> {
        let seq_id = actor.sequencer.next();
        let key = InitialKey::<M>::new();
        actor.spawn_workers(seq_id, key, self.message, ctx);
        Ok(())
    }
}

struct MessageToRoute<A: ConductedActor> {
    seq_id: SeqId,
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
        actor.spawn_workers(self.seq_id, key, self.message, ctx);
        Ok(())
    }
}

impl Pipeline {
    fn spawn_workers<K, M>(
        &mut self,
        seq_id: SeqId,
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
                let runtime = generator.generate(seq_id, pipeline, message);
                ctx.spawn_trackable(runtime, ());
            }
        }
    }
}
