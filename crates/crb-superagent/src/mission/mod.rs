pub mod async_fn;
pub mod reporting;
pub mod runtime;
pub mod sync_fn;

pub use runtime::RunMission;

use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, Context};

pub trait Goal: Send + 'static {}

impl<T> Goal for T where Self: Send + 'static {}

#[async_trait]
pub trait Mission: Agent {
    type Goal: Goal;

    async fn deliver(self, ctx: &mut Context<Self>) -> Option<Self::Goal>;
}

pub trait Observer<M: Mission>: Send {
    fn check(&mut self, goal: &M::Goal) -> Result<()>;
}
