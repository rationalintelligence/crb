use crate::runtime::{Next, StatePerformer, Transition};
use crate::agent::Agent;
use anyhow::Error;
use async_trait::async_trait;

impl<T> Next<T>
where
    T: Agent,
{
    pub fn process() -> Self {
        Self::new(ProcessPerformer)
    }
}

pub struct ProcessPerformer;

#[async_trait]
impl<T> StatePerformer<T> for ProcessPerformer
where
    T: Agent,
{
    async fn perform(&mut self, task: T, _session: &mut T::Context) -> Transition<T> {
        Transition::Process(task)
    }

    async fn fallback(&mut self, task: T, err: Error) -> (T, Next<T>) {
        (task, Next::interrupt(Some(err)))
    }
}
