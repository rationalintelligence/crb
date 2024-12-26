use crate::pipeline::PipelineState;
use crate::stage::{InitialKey, Stage, StageSource};
use async_trait::async_trait;
use std::marker::PhantomData;

pub mod stage {
    use super::*;

    pub type Input<M, S> = InputStage<M, S>;

    pub fn input<M, S>() -> InputStage<M, S> {
        InputStage::<M, S>::default()
    }
}

pub struct MessageStage<M, State> {
    message: Option<M>,
    _state: PhantomData<State>,
}

#[async_trait]
impl<M, State> Stage for MessageStage<M, State>
where
    M: Clone + Send + 'static,
    State: PipelineState,
{
    type State = State;
    type Config = ();
    type Input = M;
    type Output = M;

    fn construct(_config: Self::Config, input: Self::Input, _state: &mut Self::State) -> Self {
        Self {
            message: Some(input),
            _state: PhantomData,
        }
    }

    async fn next_output(&mut self) -> Option<Self::Output> {
        self.message.take()
    }
}

pub struct InputStage<M, State> {
    _type: PhantomData<M>,
    _state: PhantomData<State>,
}

impl<M, State> Default for InputStage<M, State> {
    fn default() -> Self {
        Self {
            _type: PhantomData,
            _state: PhantomData,
        }
    }
}

impl<M, State> StageSource for InputStage<M, State>
where
    M: Clone + Sync + Send + 'static,
    State: PipelineState,
{
    type Stage = MessageStage<M, State>;
    type Key = InitialKey<M, State>;

    fn source(&self) -> Self::Key {
        InitialKey::<M, State>::new()
    }
}
