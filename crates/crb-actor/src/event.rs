use crate::actor::Actor;
use crate::message::MessageFor;
use crate::runtime::Address;
use anyhow::Error;
use async_trait::async_trait;

impl<T> Address<T> {
    pub fn event<E>(&self, event: E) -> Result<(), Error>
    where
        T: OnEvent<E>,
        E: Send + 'static,
    {
        self.send(Event::new(event))
    }
}

#[async_trait]
pub trait OnEvent<E>: Actor {
    type Error: Into<Error> + Send + 'static;
    async fn handle(&mut self, event: E, ctx: &mut Self::Context) -> Result<(), Self::Error>;

    async fn fallback(&mut self, err: Self::Error, _ctx: &mut Self::Context) -> Result<(), Error> {
        Err(err.into())
    }
}

pub struct Event<E> {
    event: E,
}

impl<E> Event<E> {
    pub fn new(event: E) -> Self {
        Self { event }
    }
}

#[async_trait]
impl<A, E> MessageFor<A> for Event<E>
where
    A: OnEvent<E>,
    E: Send + 'static,
{
    async fn handle(self: Box<Self>, actor: &mut A, ctx: &mut A::Context) -> Result<(), Error> {
        if let Err(err) = actor.handle(self.event, ctx).await {
            actor.fallback(err, ctx).await
        } else {
            Ok(())
        }
    }
}
