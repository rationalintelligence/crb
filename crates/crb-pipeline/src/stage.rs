use crate::pipeline::{RoutePoint, RouteValue};
use async_trait::async_trait;
use std::any::type_name;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use typedmap::TypedMapKey;

#[async_trait]
pub trait Stage<LayerState = ()>: Send + 'static {
    type Config: Clone + Send;
    type Input;
    type Output: Clone + Send + 'static;

    fn construct(config: Self::Config, input: Self::Input) -> Self;
    async fn next_output(&mut self) -> Option<Self::Output>;
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
