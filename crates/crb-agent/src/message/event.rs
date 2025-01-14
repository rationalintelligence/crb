use crate::address::{Address, MessageFor};
use crate::agent::Agent;
use crate::context::Context;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_send::Recipient;

impl<A: Agent> Address<A> {
    pub fn event<E>(&self, event: E) -> Result<()>
    where
        A: OnEvent<E>,
        E: Send + 'static,
    {
        self.send(Event::new(event))
    }

    pub fn recipient<E>(&self) -> Recipient<E>
    where
        A: OnEvent<E>,
        E: Send + 'static,
    {
        Recipient::new(self.clone()).reform(Event::new)
    }
}

#[async_trait]
pub trait OnEvent<E>: Agent {
    // TODO: Add when RFC 192 will be implemented (associated types defaults)
    // type Error: Into<Error> + Send + 'static;

    async fn handle(&mut self, event: E, ctx: &mut Self::Context) -> Result<()>;

    async fn fallback(&mut self, err: Error, _ctx: &mut Self::Context) -> Result<()> {
        Err(err)
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
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        if let Err(err) = agent.handle(self.event, ctx).await {
            agent.fallback(err, ctx).await
        } else {
            Ok(())
        }
    }
}
