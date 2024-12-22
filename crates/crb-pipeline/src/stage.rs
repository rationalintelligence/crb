use crate::actor::ActorStage;
use crate::meta::Metadata;
use crate::routine::RoutineStage;
use crate::service::InputStage;
use crate::Pipeline;
use crate::{RoutePoint, RouteValue};
use anyhow::Error;
use async_trait::async_trait;
use crb_actor::MessageFor;
use crb_core::types::Clony;
use crb_supervisor::SupervisorSession;
use std::any::type_name;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use typedmap::TypedMapKey;

pub trait Stage: Send + 'static {
    type Input;
    type Output: Clone + Send + 'static;

    fn from_input(input: Self::Input) -> Self;
    fn to_output(&mut self) -> Self::Output;
}

pub trait StageSource {
    type Stage: Stage;
    type Key: TypedMapKey<Value = RouteValue<<Self::Stage as Stage>::Output>>
        + Sync
        + Send
        + 'static;
    fn source(&self) -> Self::Key;
}

pub trait StageDestination {
    type Stage: Stage;
    fn destination(&self) -> RoutePoint<<Self::Stage as Stage>::Input>;
}

pub type Input<M> = InputStage<M>;

pub fn input<M>() -> InputStage<M> {
    InputStage::<M>::default()
}

pub type Actor<A> = ActorStage<A>;

pub fn actor<A>() -> ActorStage<A> {
    ActorStage::<A>::default()
}

pub type Routine<A> = RoutineStage<A>;

pub fn routine<A>() -> RoutineStage<A> {
    RoutineStage::<A>::default()
}

pub struct InitialKey<M> {
    _type: PhantomData<M>,
}

impl<M> InitialKey<M> {
    pub fn new() -> Self {
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
    ) -> Result<(), Error> {
        let layer = actor.sequencer.next();
        let meta = Metadata::new(layer);
        let key = InitialKey::<M>::new();
        actor.spawn_workers(meta, key, self.message, ctx);
        Ok(())
    }
}

pub struct StageKey<S> {
    _type: PhantomData<S>,
}

unsafe impl<S> Sync for StageKey<S> {}

impl<S> StageKey<S> {
    pub fn new() -> Self {
        Self { _type: PhantomData }
    }
}

impl<S> Clone for StageKey<S> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<S> Hash for StageKey<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        type_name::<S>().hash(state);
    }
}

impl<S> PartialEq for StageKey<S> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<S> Eq for StageKey<S> {}

impl<S: Stage> TypedMapKey for StageKey<S> {
    type Value = RouteValue<S::Output>;
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
    ) -> Result<(), Error> {
        let key = StageKey::<A>::new();
        actor.spawn_workers(self.meta, key, self.message, ctx);
        Ok(())
    }
}
