use crate::stage::{InitialKey, Stage, StageSource};
use std::marker::PhantomData;

pub mod stage {
    use super::*;

    pub type Input<M> = InputStage<M>;

    pub fn input<M>() -> InputStage<M> {
        InputStage::<M>::default()
    }
}

pub struct MessageStage<M> {
    message: Option<M>,
}

impl<M> Stage for MessageStage<M>
where
    M: Clone + Send + 'static,
{
    type Input = M;
    type Output = M;

    fn from_input(input: Self::Input) -> Self {
        Self {
            message: Some(input),
        }
    }

    fn to_output(&mut self) -> Option<Self::Output> {
        self.message.take()
    }
}

pub struct InputStage<M> {
    _type: PhantomData<M>,
}

impl<M> Default for InputStage<M> {
    fn default() -> Self {
        Self { _type: PhantomData }
    }
}

impl<M> StageSource for InputStage<M>
where
    M: Clone + Sync + Send + 'static,
{
    type Stage = MessageStage<M>;
    type Key = InitialKey<M>;

    fn source(&self) -> Self::Key {
        InitialKey::<M>::new()
    }
}
