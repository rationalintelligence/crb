use async_trait::async_trait;
use crb_agent::{Address, Agent, AgentContext, AgentSession, Envelope, Event, OnEvent, TheEvent};
use crb_runtime::{ManagedContext, ReachableContext};
use derive_more::{Deref, DerefMut};
use futures::{future::select, Stream, StreamExt};
use futures_util::stream::SelectAll;

pub trait IntoEvents<E> {
    type Stream: Stream<Item = E> + Unpin + Send + 'static;

    fn into_events(self) -> Self::Stream;
}

pub trait EnvelopeStream<A>: Stream<Item = Envelope<A>> + Unpin + Send + 'static {}

impl<A, T> EnvelopeStream<A> for T where Self: Stream<Item = Envelope<A>> + Unpin + Send + 'static {}

#[derive(Deref, DerefMut)]
pub struct StreamSession<A: Agent> {
    #[deref]
    #[deref_mut]
    session: AgentSession<A>,
    streams: SelectAll<Box<dyn EnvelopeStream<A>>>,
}

impl<A: Agent> Default for StreamSession<A> {
    fn default() -> Self {
        Self {
            session: AgentSession::default(),
            streams: SelectAll::default(),
        }
    }
}

impl<A: Agent> ReachableContext for StreamSession<A> {
    type Address = Address<A>;

    fn address(&self) -> &Self::Address {
        self.session.address()
    }
}

impl<A: Agent> ManagedContext for StreamSession<A> {
    fn is_alive(&self) -> bool {
        self.session.is_alive()
    }

    fn shutdown(&mut self) {
        self.session.shutdown()
    }

    fn stop(&mut self) {
        self.session.stop();
    }
}

#[async_trait]
impl<A: Agent> AgentContext<A> for StreamSession<A> {
    fn session(&mut self) -> &mut AgentSession<A> {
        &mut self.session
    }

    async fn next_envelope(&mut self) -> Option<Envelope<A>> {
        let next_fut = self.session.next_envelope();
        if self.streams.is_empty() {
            next_fut.await
        } else {
            let event = self.streams.next();
            select(next_fut, event).await.factor_first().0
        }
    }
}

impl<A: Agent> StreamSession<A> {
    // TODO: Consume streams here
    pub fn consume<E>(&mut self, stream: impl IntoEvents<E>)
    where
        A: OnEvent<E>,
        E: TheEvent,
    {
        let stream = stream.into_events().map(Event::envelope::<A>);
        self.streams.push(Box::new(stream));
    }
}
