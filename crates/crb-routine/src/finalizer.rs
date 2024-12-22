use crate::TaskError;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_actor::kit::{Actor, Address, MessageFor};

pub type BoxFinalizer<O> = Box<dyn Finalizer<O>>;

#[async_trait]
pub trait Finalizer<O>
where
    Self: Send + 'static,
    O: Send + 'static,
{
    async fn finalize(&mut self, output: Result<O, TaskError>) -> Result<()> {
        output?;
        Ok(())
    }
}

#[async_trait]
pub trait OnOutput<O>: Actor {
    async fn on_output(&mut self, output: Result<O, TaskError>) -> Result<()>;
}

#[async_trait]
impl<A, O> Finalizer<O> for Address<A>
where
    A: OnOutput<O>,
    O: Send + 'static,
{
    async fn finalize(&mut self, output: Result<O, TaskError>) -> Result<()> {
        let msg = RoutineOutput { output };
        self.send(msg)?;
        Ok(())
    }
}

struct RoutineOutput<O> {
    output: Result<O, TaskError>,
}

#[async_trait]
impl<A, O> MessageFor<A> for RoutineOutput<O>
where
    A: OnOutput<O>,
    O: Send + 'static,
{
    async fn handle(self: Box<Self>, actor: &mut A, _ctx: &mut A::Context) -> Result<(), Error> {
        actor.on_output(self.output).await
    }
}
