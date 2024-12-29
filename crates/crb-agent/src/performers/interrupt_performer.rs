use crate::agent::Agent;
use crate::performers::{Next, StatePerformer, Transition, TransitionCommand};
use anyhow::Error;
use async_trait::async_trait;

impl<T> Next<T>
where
    T: Agent,
{
    pub fn done() -> Self {
        Self::interrupt(None)
    }

    pub fn fail(err: Error) -> Self {
        Self::interrupt(Some(err))
    }

    pub(crate) fn interrupt(error: Option<Error>) -> Self {
        Self::new(InterruptPerformer { error })
    }
}

pub struct InterruptPerformer {
    error: Option<Error>,
}

#[async_trait]
impl<T> StatePerformer<T> for InterruptPerformer
where
    T: Agent,
{
    async fn perform(&mut self, agent: T, _session: &mut T::Context) -> Transition<T> {
        match self.error.take() {
            None => {
                let command = TransitionCommand::Interrupted;
                Transition::Continue { agent, command }
            }
            Some(err) => Transition::Crashed(err),
        }
    }

    async fn fallback(&mut self, agent: T, err: Error) -> (T, Next<T>) {
        let error = self.error.take().unwrap_or(err);
        (agent, Next::interrupt(Some(error)))
    }
}
