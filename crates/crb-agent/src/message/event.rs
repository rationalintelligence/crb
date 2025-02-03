use crate::address::{Address, Envelope, MessageFor};
use crate::agent::Agent;
use crate::context::Context;
use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use crb_core::Tag;
use crb_send::Recipient;

pub trait EventExt<E: TheEvent> {
    fn event(&self, event: E) -> Result<()>;
}

impl<A, E> EventExt<E> for Address<A>
where
    A: OnEvent<E>,
    E: TheEvent,
{
    fn event(&self, event: E) -> Result<()> {
        Address::event(self, event)
    }
}

impl<A, E> EventExt<E> for Context<A>
where
    A: OnEvent<E>,
    E: TheEvent,
{
    fn event(&self, event: E) -> Result<()> {
        self.address().event(event)
    }
}

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

    pub fn event_tagged<E, T>(&self, event: E, tag: T) -> Result<()>
    where
        A: OnEvent<E, T>,
        E: TheEvent,
        T: Tag,
    {
        self.send(Event::new_tagged(event, tag))
    }

    pub fn recipient<E>(&self) -> Recipient<E>
    where
        A: OnEvent<E>,
        E: TheEvent,
    {
        Recipient::new(self.clone()).reform(Event::new)
    }

    pub fn recipient_tagged<E, T>(&self, tag: T) -> Recipient<E>
    where
        A: OnEvent<E, T>,
        E: TheEvent,
        T: Tag + Sync + Clone,
    {
        Recipient::new(self.clone()).reform(move |event| Event::new_tagged(event, tag.clone()))
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

    pub fn event_tagged<E, T>(&self, event: E, tag: T) -> Result<()>
    where
        A: OnEvent<E, T>,
        E: TheEvent,
        T: Tag,
    {
        self.address().event_tagged(event, tag)
    }

    pub fn recipient<E>(&self) -> Recipient<E>
    where
        A: OnEvent<E>,
        E: TheEvent,
    {
        self.address().recipient()
    }

    pub fn recipient_tagged<E, T>(&self, tag: T) -> Recipient<E>
    where
        A: OnEvent<E, T>,
        E: TheEvent,
        T: Tag + Sync + Clone,
    {
        self.address().recipient_tagged(tag)
    }
}

/// Do not introduce tags: use event wrapper instead.
#[async_trait]
pub trait OnEvent<E: TheEvent, T: Tag = ()>: Agent {
    // TODO: Add when RFC 192 will be implemented (associated types defaults)
    // type Error: Into<Error> + Send + 'static;

    async fn handle_tagged(&mut self, event: E, _tag: T, ctx: &mut Context<Self>) -> Result<()> {
        self.handle(event, ctx).await
    }

    async fn handle(&mut self, _event: E, _ctx: &mut Context<Self>) -> Result<()> {
        Err(anyhow!("The handle method in not implemented."))
    }

    async fn fallback(&mut self, err: Error, _ctx: &mut Context<Self>) -> Result<()> {
        Err(err)
    }
}

pub struct Event<E, T = ()> {
    event: E,
    tag: T,
}

impl<E> Event<E> {
    pub fn new(event: E) -> Self {
        Self { event, tag: () }
    }

    pub fn envelope<A>(event: E) -> Envelope<A>
    where
        A: OnEvent<E>,
        E: TheEvent,
    {
        Self::new(event).into_envelope::<A>()
    }
}

impl<E, T> Event<E, T> {
    pub fn new_tagged(event: E, tag: T) -> Self {
        Self { event, tag }
    }
}

impl<E, T> Event<E, T>
where
    E: TheEvent,
    T: Tag,
{
    pub fn into_envelope<A>(self) -> Envelope<A>
    where
        A: OnEvent<E, T>,
    {
        Box::new(self)
    }
}

#[async_trait]
impl<A, E, T> MessageFor<A> for Event<E, T>
where
    A: OnEvent<E, T>,
    E: TheEvent,
    T: Tag,
{
    async fn handle(self: Box<Self>, agent: &mut A, ctx: &mut Context<A>) -> Result<()> {
        if let Err(err) = agent.handle_tagged(self.event, self.tag, ctx).await {
            agent.fallback(err, ctx).await
        } else {
            Ok(())
        }
    }
}
