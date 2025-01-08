use crate::agent::Agent;
use crate::performers::{Next, StatePerformer, Transition, TransitionCommand};
use async_trait::async_trait;

impl<A> Next<A>
where
    A: Agent,
{
    pub fn events() -> Self {
        Self::new(EventsPerformer)
    }
}

pub struct EventsPerformer;

#[async_trait]
impl<A> StatePerformer<A> for EventsPerformer
where
    A: Agent,
{
    async fn perform(&mut self, agent: A, _session: &mut A::Context) -> Transition<A> {
        let command = TransitionCommand::ProcessEvents;
        Transition::Continue { agent, command }
    }
}
