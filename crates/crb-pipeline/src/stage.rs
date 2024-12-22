use crate::{ActorRuntimeGenerator, ConductedActor, RoutePoint};
use std::marker::PhantomData;

pub trait Stage {
    type Input;
    type Output;

    fn from_input(input: Self::Input) -> Self;
    fn to_output(self) -> Self::Output;
}

pub struct Actor<A> {
    _type: PhantomData<A>,
}

impl<A> Actor<A> {
    pub fn stage() -> Self {
        Self { _type: PhantomData }
    }
}

impl<A, IN, OUT> StageRoute<IN, OUT> for Actor<A>
where
    A: ConductedActor<Input = IN, Output = OUT>,
    A: Stage<Input = IN, Output = OUT>,
    IN: 'static,
{
    fn source(&self) {}

    fn recipient(&self) -> RoutePoint<IN> {
        let generator = ActorRuntimeGenerator::<A>::new::<IN>();
        Box::new(generator)
    }
}

pub trait StageRoute<IN, OUT> {
    fn source(&self);
    fn recipient(&self) -> RoutePoint<IN>;
}

/*
impl<A, IN, OUT> Stage<IN, OUT> for Actor<A> {
}

pub trait Stage<IN, OUT> {
    /*
    fn from_input(input: IN) -> Self;
    fn to_output(self) -> OUT;
    */
}

/*
impl<T, IN, OUT> Stage<IN, OUT> for T
where
    T: From<IN>,
    T: Into<OUT>,
{
    fn from_input(input: IN) -> Self {
        input.into()
    }

    fn to_output(self) -> OUT {
        self.into()
    }
}
*/
*/
