use crate::{ActorRuntimeGenerator, ConductedActor, RouteKey, RoutePoint, RouteValue};
use std::marker::PhantomData;
use typedmap::TypedMapKey;

pub trait Stage {
    type Input;
    type Output;

    fn from_input(input: Self::Input) -> Self;
    fn to_output(self) -> Self::Output;
}

pub trait StageSource {
    type Stage: Stage;
    type Key;
    fn source(&self) -> Self::Key;
}

pub trait StageDestination {
    type Stage: Stage;
    fn destination(&self) -> RoutePoint<<Self::Stage as Stage>::Input>;
}

pub struct Actor<A> {
    _type: PhantomData<A>,
}

impl<A> Actor<A> {
    pub fn stage() -> Self {
        Self { _type: PhantomData }
    }
}

impl<A> StageSource for Actor<A>
where
    A: Stage,
{
    type Stage = A;
    type Key = RouteKey<A>;

    fn source(&self) -> Self::Key {
        RouteKey::<A>::new()
    }
}

impl<A> StageDestination for Actor<A>
where
    A: Stage,
{
    type Stage = A;

    fn destination(&self) -> RoutePoint<A::Input> {
        /*
        let generator = ActorRuntimeGenerator::<A>::new::<A::Input>();
        Box::new(generator)
        */
        todo!()
    }
}
