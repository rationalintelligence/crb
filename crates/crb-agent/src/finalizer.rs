use crate::agent::Agent;
use anyhow::Result;

pub trait FinalizerFor<A: Agent>: Send {
    fn finalize(&mut self, output: &A::Output) -> Result<()>;
}
