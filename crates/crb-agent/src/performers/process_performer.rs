use crate::agent::Agent;
use crate::performers::{Next, StatePerformer, Transition, TransitionCommand};
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
    async fn perform(&mut self, agent: T, _session: &mut T::Context) -> Transition<T> {
        let command = TransitionCommand::Process;
        Transition::Continue { agent, command }
    }
}
