pub mod async_fn;
pub mod reporting;
pub mod runtime;
pub mod sync_fn;

pub use runtime::RunMission;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::{Agent, Context};
use std::any::type_name;

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

#[async_trait]
pub trait Operable: Mission {
    async fn operate(self) -> Result<Self::Goal>;
}

#[async_trait]
impl<M: Mission> Operable for M
where
    Self::Context: Default,
{
    async fn operate(self) -> Result<M::Goal> {
        let mut runtime = RunMission::new(self);
        runtime
            .perform()
            .await
            .ok_or_else(|| anyhow!("Mission {} failed", type_name::<Self>()))
    }
}
