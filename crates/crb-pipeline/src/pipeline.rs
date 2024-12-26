use crate::meta::{Metadata, Sequencer};
use crate::stage::{InitialKey, Stage, StageDestination, StageKey, StageSource};
use anyhow::Result;
use async_trait::async_trait;
use crb_actor::kit::{Actor, Address, MessageFor};
use crb_core::types::Clony;
use crb_runtime::kit::{Context, Runtime};
use crb_supervisor::{Supervisor, SupervisorSession};
use derive_more::Deref;
use std::any::type_name;
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

    pub fn route<FROM, TO>(&mut self)
    where
        FROM: StageSource + Default,
        TO: StageDestination + Default,
        TO::Stage: Stage<Input = <FROM::Stage as Stage>::Output>,
    {
        self.stage(FROM::default(), TO::default())
    }
}

impl Supervisor for Pipeline {
    type GroupBy = ();
}

impl Actor for Pipeline {
    type Context = SupervisorSession<Self>;
}

#[derive(Deref)]
pub struct RoutePoint<M> {
    pub generator: Box<dyn RuntimeGenerator<Input = M>>,
}

impl<M> RoutePoint<M> {
    pub fn new(generator: impl RuntimeGenerator<Input = M>) -> Self {
        Self {
            generator: Box::new(generator),
        }
    }
}

pub type RouteValue<M> = Vec<RoutePoint<M>>;

pub trait RuntimeGenerator: Send + Sync + 'static {
    type Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline>,
        input: Self::Input,
    ) -> Box<dyn Runtime>;
}

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
        let mut spawned = 0;
        let generators = self.routes.get(&key);
        if let Some(generators) = generators {
            for generator in generators.iter() {
                let pipeline = ctx.address().clone();
                let message = message.clone();
                let runtime = generator.generate(meta, pipeline, message);
                ctx.spawn_trackable(runtime, ());
                spawned += 1;
            }
        }
        if spawned == 0 {
            log::error!(
                "Workers for {} are not presented. Source: {}",
                type_name::<M>(),
                type_name::<K>()
            );
        }
    }
}

pub struct InitialMessage<M> {
    message: M,
}

impl<M> InitialMessage<M> {
    pub fn new(message: M) -> Self {
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
    ) -> Result<()> {
        let layer = actor.sequencer.next();
        let meta = Metadata::new(layer);
        let key = InitialKey::<M>::new();
        actor.spawn_workers(meta, key, self.message, ctx);
        Ok(())
    }
}

pub struct StageReport<S: Stage> {
    meta: Metadata,
    message: S::Output,
}

impl<S: Stage> StageReport<S> {
    pub fn new(meta: Metadata, message: S::Output) -> Self {
        Self { meta, message }
    }
}

#[async_trait]
impl<A> MessageFor<Pipeline> for StageReport<A>
where
    A: Stage,
{
    async fn handle(
        self: Box<Self>,
        actor: &mut Pipeline,
        ctx: &mut SupervisorSession<Pipeline>,
    ) -> Result<()> {
        let key = StageKey::<A>::new();
        actor.spawn_workers(self.meta, key, self.message, ctx);
        Ok(())
    }
}
