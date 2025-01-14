use crate::address::{Address, MessageFor};
use crate::agent::Agent;
use crate::context::Context;
use crate::finalizer::FinalizerFor;
use crate::runtime::RunAgent;
use anyhow::Result;
use async_trait::async_trait;

impl<A: Agent> RunAgent<A> {
    pub fn report_to<S>(&mut self, address: impl AsRef<Address<S>>)
    where
        S: Finished<A>,
        A::Output: Clone,
    {
        let address = address.as_ref().clone();
        let finalizer = Box::new(address);
        self.finalizers.push(finalizer);
    }
}

impl<S, A> FinalizerFor<A> for Address<S>
where
    S: Finished<A>,
    A: Agent,
    A::Output: Clone,
{
    fn finalize(&mut self, output: &A::Output) -> Result<()> {
        let output = output.clone();
        let event = FinishedEvent { output };
        self.send(event)
    }
}

#[async_trait]
pub trait Finished<A: Agent>: Agent {
    async fn handle(&mut self, output: A::Output, ctx: &mut Self::Context) -> Result<()>;
}

struct FinishedEvent<A: Agent> {
    output: A::Output,
}

#[async_trait]
impl<S, A> MessageFor<S> for FinishedEvent<A>
where
    S: Finished<A>,
    A: Agent,
{
    async fn handle(self: Box<Self>, agent: &mut S, ctx: &mut Context<S>) -> Result<()> {
        agent.handle(self.output, ctx).await
    }
}
