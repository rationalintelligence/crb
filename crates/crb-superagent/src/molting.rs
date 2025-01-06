use anyhow::Error;
use async_trait::async_trait;
use crb_agent::performers::{Next, StatePerformer, StopReason, Transition, TransitionCommand};
use crb_agent::Agent;

pub trait MoltTo<T> {
    fn molt(self) -> T;
}

pub struct MoltPerformer {
    reason: Option<StopReason>,
}

#[async_trait]
impl<A> StatePerformer<A> for MoltPerformer
where
    A: Agent,
{
    async fn perform(&mut self, agent: A, _session: &mut A::Context) -> Transition<A> {
        todo!()
    }
}
