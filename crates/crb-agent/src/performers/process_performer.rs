use crate::agent::Agent;
use crate::performers::{Next, StatePerformer, Transition, TransitionCommand};
use async_trait::async_trait;

impl<A> Next<A>
where
    A: Agent,
{
    pub fn process() -> Self {
        Self::new(ProcessPerformer)
    }
}

pub struct ProcessPerformer;

#[async_trait]
impl<A> StatePerformer<A> for ProcessPerformer
where
    A: Agent,
{
    async fn perform(&mut self, agent: A, _session: &mut A::Context) -> Transition<A> {
        let command = TransitionCommand::Process;
        Transition::Continue { agent, command }
    }
}
