use crate::runtime::{Next, StatePerformer, Transition};
use crate::agent::Agent;
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
    async fn perform(&mut self, task: T, _session: &mut T::Context) -> Transition<T> {
        match self.error.take() {
            None => Transition::Interrupted(task),
            Some(err) => Transition::Crashed(err),
        }
    }

    async fn fallback(&mut self, task: T, err: Error) -> (T, Next<T>) {
        let error = self.error.take().unwrap_or(err);
        (task, Next::interrupt(Some(error)))
    }
}
