use crate::context::AgentContext;
use crate::runtime::NextState;
use anyhow::{Error, Result};
use async_trait::async_trait;

pub trait Agent: Sized + Send + 'static {
    type Output: Default + Clone;
    type Context: AgentContext<Self>;

    fn initialize(&mut self, _ctx: &mut Self::Context) -> NextState<Self> {
        NextState::process()
    }


    fn finalize(&mut self, _ctx: &mut Self::Context) -> Self::Output {
        Self::Output::default()
    }

    // TODO: Add finalizers
    // type Output: Default;

}
