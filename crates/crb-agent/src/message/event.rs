use crate::address::{Address, MessageFor};
use crate::agent::Agent;
use crate::context::Context;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_send::Recipient;

pub trait TheEvent: Send + 'static {}

impl<T> TheEvent for T where Self: Send + 'static {}

impl<A: Agent> Address<A> {
    pub fn event<E>(&self, event: E) -> Result<()>
    where
        A: OnEvent<E>,
        E: TheEvent,
    {
        self.send(Event::new(event))
    }

    pub fn recipient<E>(&self) -> Recipient<E>
    where
        A: OnEvent<E>,
        E: TheEvent,
    {
        Recipient::new(self.clone()).reform(Event::new)
    }
}

impl<A: Agent> Context<A> {
    pub fn event<E>(&self, event: E) -> Result<()>
    where
        A: OnEvent<E>,
        E: TheEvent,
    {
        self.address().event(event)
    }

    pub fn recipient<E>(&self) -> Recipient<E>
    where
        A: OnEvent<E>,
        E: TheEvent,
    {
        self.address().recipient()
    }
}

/// Do not introduce tags: use event wrapper instead.
#[async_trait]
pub trait OnEvent<E>: Agent {
    // TODO: Add when RFC 192 will be implemented (associated types defaults)
    // type Error: Into<Error> + Send + 'static;

    async fn handle(&mut self, event: E, ctx: &mut Context<Self>) -> Result<()>;

    async fn fallback(&mut self, err: Error, _ctx: &mut Context<Self>) -> Result<()> {
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
    E: TheEvent,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        if let Err(err) = agent.handle(self.event, ctx).await {
            agent.fallback(err, ctx).await
        } else {
            Ok(())
        }
    }
}
