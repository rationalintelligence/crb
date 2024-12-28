use crate::meta::{Metadata, Sequencer};
use crate::stage::{InitialKey, Stage, StageDestination, StageKey, StageSource};
use anyhow::Result;
use async_trait::async_trait;
use crb_actor::kit::{Actor, Address, MessageFor, Standalone};
use crb_core::types::Clony;
use crb_runtime::kit::{Context, Runtime};
use crb_supervisor::{Supervisor, SupervisorSession};
use derive_more::Deref;
use std::any::type_name;
use std::marker::PhantomData;
use typedmap::{TypedDashMap, TypedMapKey};

pub trait PipelineState: Sync + Send + 'static {}

impl<T> PipelineState for T where T: Sync + Send + 'static {}

pub struct Pipeline<State: PipelineState = ()> {
    sequencer: Sequencer,
    routes: TypedDashMap,
    state: State,
}

impl<State: PipelineState> Pipeline<State> {
    pub fn new() -> Self
    where
        State: Default,
    {
        Self {
            sequencer: Sequencer::default(),
            routes: TypedDashMap::default(),
            state: State::default(),
        }
    }

    pub fn with_state(state: State) -> Self {
        Self {
            sequencer: Sequencer::default(),
            routes: TypedDashMap::default(),
            state,
        }
    }

    pub fn stage<FROM, TO>(&mut self, from: FROM, to: TO)
    where
        FROM: StageSource,
        FROM::Stage: Stage<State = State>,
        TO: StageDestination,
        TO::Stage: Stage<Input = <FROM::Stage as Stage>::Output, State = State>,
    {
        let key = from.source();
        let generator = to.destination();
        self.routes.entry(key).or_default().push(generator);
    }

    pub fn route<FROM, TO>(&mut self)
    where
        FROM: StageSource + Default,
        FROM::Stage: Stage<State = State>,
        TO: StageDestination + Default,
        TO::Stage: Stage<Input = <FROM::Stage as Stage>::Output, State = State>,
    {
        self.stage(FROM::default(), TO::default())
    }
}

impl<State: PipelineState> Supervisor for Pipeline<State> {
    type GroupBy = ();
}

impl<State: PipelineState> Standalone for Pipeline<State> {}

impl<State: PipelineState> Actor for Pipeline<State> {
    type Context = SupervisorSession<Self>;
}

#[derive(Deref)]
pub struct RoutePoint<M, State> {
    #[deref]
    generator: Box<dyn RuntimeGenerator<Input = M, State = State>>,
    _state: PhantomData<State>,
}

impl<M, State> RoutePoint<M, State> {
    pub fn new(generator: impl RuntimeGenerator<Input = M, State = State>) -> Self {
        Self {
            generator: Box::new(generator),
            _state: PhantomData,
        }
    }
}

pub type RouteValue<M, S> = Vec<RoutePoint<M, S>>;

pub trait RuntimeGenerator: Send + Sync + 'static {
    type State: PipelineState;
    type Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline<Self::State>>,
        input: Self::Input,
        state: &mut Self::State,
    ) -> Box<dyn Runtime>;
}

impl<State: PipelineState> Pipeline<State> {
    fn spawn_workers<K, M>(
        &mut self,
        meta: Metadata,
        key: K,
        message: M,
        ctx: &mut SupervisorSession<Self>,
    ) where
        K: TypedMapKey<Value = RouteValue<M, State>> + Send + Sync + 'static,
        M: Clone + 'static,
    {
        let mut spawned = 0;
        let generators = self.routes.get(&key);
        if let Some(generators) = generators {
            for generator in generators.iter() {
                let pipeline = ctx.address().clone();
                let message = message.clone();
                let runtime = generator.generate(meta, pipeline, message, &mut self.state);
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
impl<M, State> MessageFor<Pipeline<State>> for InitialMessage<M>
where
    M: Clony,
    State: PipelineState,
{
    async fn handle(
        self: Box<Self>,
        actor: &mut Pipeline<State>,
        ctx: &mut SupervisorSession<Pipeline<State>>,
    ) -> Result<()> {
        let layer = actor.sequencer.next();
        let meta = Metadata::new(layer);
        let key = InitialKey::<M, State>::new();
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
impl<A> MessageFor<Pipeline<A::State>> for StageReport<A>
where
    A: Stage,
{
    async fn handle(
        self: Box<Self>,
        actor: &mut Pipeline<A::State>,
        ctx: &mut SupervisorSession<Pipeline<A::State>>,
    ) -> Result<()> {
        let key = StageKey::<A>::new();
        actor.spawn_workers(self.meta, key, self.message, ctx);
        Ok(())
    }
}
