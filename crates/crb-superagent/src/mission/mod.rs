pub mod fn_agent;
pub mod reporting;
pub mod runtime;

use anyhow::Result;
use crb_agent::{Agent, Context};

pub trait Goal: Send + 'static {}

impl<T> Goal for T where Self: Send + 'static {}

pub trait Mission: Agent {
    type Goal: Goal;

    fn deliver(self, ctx: &mut Context<Self>) -> Option<Self::Goal>;
}

pub trait Observer<M: Mission>: Send {
    fn check(&mut self, goal: &M::Goal) -> Result<()>;
}
