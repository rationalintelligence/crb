use anyhow::Error;
use async_trait::async_trait;
use crb_core::{mpsc, watch};
use crb_runtime::{Context, Controller, Failures, Interruptor, ManagedContext, Runtime};

pub trait MorphContext: Context + 'static {
    fn morph(&mut self, next: impl Morph<Context = Self>);
    fn next_state(&mut self) -> Option<Box<dyn Morph<Context = Self>>>;
}

pub struct MorphSession {
    next_state: Option<Box<dyn Morph<Context = Self>>>,
}

impl MorphContext for MorphSession {
    fn morph(&mut self, next: impl Morph<Context = Self>) {
        self.next_state = Some(Box::new(next));
    }

    fn next_state(&mut self) -> Option<Box<dyn Morph<Context = Self>>> {
        self.next_state.take()
    }
}

impl Context for MorphSession {
    type Address = ();

    fn address(&self) -> &Self::Address {
        &()
    }
}

pub trait Morph: Send + 'static {
    // TODO:
    // type Message; ? and the `send` method that expects `impl Into<Morph::Message>`
    // Routing? call the method only if that is implemented for a particular state?
    type Context;
}

pub struct MorphRuntime<C> {
    context: C,
    morphed: Box<dyn Morph<Context = C>>,
}

#[async_trait]
impl<C> Runtime for MorphRuntime<C>
where
    C: MorphContext,
{
    type Context = C;

    fn get_interruptor(&mut self) -> Interruptor {
        todo!()
    }

    fn address(&self) -> <Self::Context as Context>::Address {
        self.context.address().clone()
    }

    async fn routine(&mut self) {
        loop {
            if let Some(morphed) = self.context.next_state() {
                self.morphed = morphed;
            }
        }
    }
}
