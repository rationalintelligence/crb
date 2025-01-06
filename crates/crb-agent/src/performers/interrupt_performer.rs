use crate::agent::Agent;
use crate::performers::{Next, StatePerformer, StopReason, Transition, TransitionCommand};
use anyhow::Error;
use async_trait::async_trait;

impl<A> Next<A>
where
    A: Agent,
{
    pub fn done() -> Self {
        Self::stop(StopReason::Done)
    }

    pub fn interrupt() -> Self {
        Self::stop(StopReason::Interrupted)
    }

    pub fn fail(err: Error) -> Self {
        Self::stop(StopReason::Failed(err))
    }

    pub fn todo(reason: impl ToString) -> Self {
        let err = Error::msg(reason.to_string());
        Self::stop(StopReason::Failed(err))
    }

    pub(crate) fn stop(reason: StopReason) -> Self {
        Self::new(StopPerformer {
            reason: Some(reason),
        })
    }
}

pub struct StopPerformer {
    reason: Option<StopReason>,
}

#[async_trait]
impl<A> StatePerformer<A> for StopPerformer
where
    A: Agent,
{
    async fn perform(&mut self, agent: A, _session: &mut A::Context) -> Transition<A> {
        let reason = self.reason.take().unwrap_or(StopReason::Interrupted);
        let command = TransitionCommand::Stop(reason);
        Transition::Continue { agent, command }
    }
}
