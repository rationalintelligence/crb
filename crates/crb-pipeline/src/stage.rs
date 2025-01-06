use crate::pipeline::{PipelineState, RoutePoint, RouteValue};
use async_trait::async_trait;
use std::any::type_name;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use typedmap::TypedMapKey;

#[async_trait]
pub trait Stage: Send + 'static {
    type State: PipelineState;
    type Config: Clone + Send;
    type Input;
    type Output: Clone + Send + 'static;

    fn construct(config: Self::Config, input: Self::Input, state: &mut Self::State) -> Self;
    async fn next_output(&mut self) -> Option<Self::Output>;
}

pub trait StageSource {
    type Stage: Stage;
    type Key: TypedMapKey<
            Value = RouteValue<<Self::Stage as Stage>::Output, <Self::Stage as Stage>::State>,
        > + Sync
        + Send
        + 'static;
    fn source(&self) -> Self::Key;
}

pub trait StageDestination {
    type Stage: Stage;
    fn destination(
        &self,
    ) -> RoutePoint<<Self::Stage as Stage>::Input, <Self::Stage as Stage>::State>;
}

pub struct InitialKey<M, State> {
    _type: PhantomData<M>,
    _state: PhantomData<State>,
}

impl<M, State> Default for InitialKey<M, State> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M, State> InitialKey<M, State> {
    pub fn new() -> Self {
        Self {
            _type: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<M, State> Clone for InitialKey<M, State> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<M, State> Hash for InitialKey<M, State> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        type_name::<M>().hash(state);
    }
}

impl<M, State> PartialEq for InitialKey<M, State> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<M, State> Eq for InitialKey<M, State> {}

impl<M: 'static, State: PipelineState> TypedMapKey for InitialKey<M, State> {
    type Value = RouteValue<M, State>;
}

pub struct StageKey<S> {
    _type: PhantomData<S>,
}

unsafe impl<S> Sync for StageKey<S> {}

impl<S> Default for StageKey<S> {
    fn default() -> Self {
        Self::new()
    }
}

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
    type Value = RouteValue<S::Output, S::State>;
}
