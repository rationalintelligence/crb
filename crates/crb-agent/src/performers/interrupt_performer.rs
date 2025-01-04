use crate::agent::Agent;
use crate::performers::{Next, StatePerformer, Transition, TransitionCommand};
use anyhow::Error;
use async_trait::async_trait;

impl<A> Next<A>
where
    A: Agent,
{
    pub fn done() -> Self {
        Self::stop(TransitionCommand::Done)
    }

    pub fn interrupt() -> Self {
        Self::stop(TransitionCommand::Interrupted)
    }

    pub fn fail(err: Error) -> Self {
        Self::stop(TransitionCommand::Failed(err))
    }

    pub(crate) fn stop(command: TransitionCommand<A>) -> Self {
        Self::new(StopPerformer {
            command: Some(command),
        })
    }
}

pub struct StopPerformer<A: Agent> {
    command: Option<TransitionCommand<A>>,
}

#[async_trait]
impl<A> StatePerformer<A> for StopPerformer<A>
where
    A: Agent,
{
    async fn perform(&mut self, agent: A, _session: &mut A::Context) -> Transition<A> {
        let command = self
            .command
            .take()
            .unwrap_or(TransitionCommand::Interrupted);
        Transition::Continue { agent, command }
    }
}
