use crate::address::{Address, MessageFor};
use crate::agent::Agent;
use anyhow::{Error, Result};
use async_trait::async_trait;

impl<T: Agent> Address<T> {
    pub fn event<E>(&self, event: E) -> Result<()>
    where
        T: OnEvent<E>,
        E: Send + 'static,
    {
        self.send(Event::new(event))
    }
}

#[async_trait]
pub trait OnEvent<E>: Agent {
    type Error: Into<Error> + Send + 'static;
    async fn handle(&mut self, event: E, ctx: &mut Self::Context) -> Result<(), Self::Error>;

    async fn fallback(&mut self, err: Self::Error, _ctx: &mut Self::Context) -> Result<()> {
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
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut A::Context) -> Result<()> {
        if let Err(err) = agent.handle(self.event, ctx).await {
            agent.fallback(err, ctx).await
        } else {
            Ok(())
        }
    }
}
