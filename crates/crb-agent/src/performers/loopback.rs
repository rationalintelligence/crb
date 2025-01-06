use crate::address::Envelope;
use crate::agent::Agent;
use crate::message::loopback::{InContext, LoopbackEvent};
use crate::performers::{Next, StatePerformer, Transition, TransitionCommand};
use async_trait::async_trait;

impl<T> Next<T>
where
    T: Agent,
{
    pub fn in_context<E>(event: E) -> Self
    where
        T: InContext<E>,
        E: Send + 'static,
    {
        let event = LoopbackEvent::new(event);
        Self::new(Loopback {
            envelope: Some(Box::new(event)),
        })
    }
}

pub struct Loopback<T> {
    envelope: Option<Envelope<T>>,
}

#[async_trait]
impl<T> StatePerformer<T> for Loopback<T>
where
    T: Agent,
{
    async fn perform(&mut self, agent: T, _session: &mut T::Context) -> Transition<T> {
        let envelope = self
            .envelope
            .take()
            .expect("Loopback performer must be called once");
        let command = TransitionCommand::InContext(envelope);
        Transition::Continue { agent, command }
    }
}
