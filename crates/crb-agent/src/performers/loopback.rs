use crate::address::Envelope;
use crate::agent::Agent;
use crate::message::loopback::{InContext, LoopbackEvent};
use crate::performers::{Next, StatePerformer, Transition, TransitionCommand};
use async_trait::async_trait;

impl<A> Next<A>
where
    A: Agent,
{
    pub fn in_context<E>(event: E) -> Self
    where
        A: InContext<E>,
        E: Send + 'static,
    {
        let event = LoopbackEvent::new(event);
        Self::new(Loopback {
            envelope: Some(Box::new(event)),
        })
    }
}

pub struct Loopback<A> {
    envelope: Option<Envelope<A>>,
}

#[async_trait]
impl<A> StatePerformer<A> for Loopback<A>
where
    A: Agent,
{
    async fn perform(&mut self, agent: A, _session: &mut A::Context) -> Transition<A> {
        let envelope = self
            .envelope
            .take()
            .expect("Loopback performer must be called once");
        let command = TransitionCommand::InContext(envelope);
        Transition::Continue { agent, command }
    }
}
