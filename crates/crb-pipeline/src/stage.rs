use crate::{RoutePoint, RouteValue};
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
