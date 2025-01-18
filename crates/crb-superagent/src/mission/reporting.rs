use super::runtime::RunMission;
use super::{Mission, Observer};
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Address, Agent, Context, MessageFor, ToAddress};

impl<M: Mission> RunMission<M> {
    pub fn report_to<S>(&mut self, address: impl ToAddress<S>)
    where
        S: Finished<M>,
        M::Goal: Clone,
    {
        let address = address.to_address();
        let observer = Box::new(address);
        self.observers.push(observer);
    }
}

impl<S, M> Observer<M> for Address<S>
where
    S: Finished<M>,
    M: Mission,
    M::Goal: Clone,
{
    fn check(&mut self, output: &M::Goal) -> Result<()> {
        let output = output.clone();
        let event = FinishedEvent { output };
        self.send(event)
    }
}

#[async_trait]
pub trait Finished<M: Mission>: Agent {
    async fn handle(&mut self, output: M::Goal, ctx: &mut Context<Self>) -> Result<()>;
}

struct FinishedEvent<M: Mission> {
    output: M::Goal,
}

#[async_trait]
impl<S, M> MessageFor<S> for FinishedEvent<M>
where
    S: Finished<M>,
    M: Mission,
{
    async fn handle(self: Box<Self>, agent: &mut S, ctx: &mut Context<S>) -> Result<()> {
        agent.handle(self.output, ctx).await
    }
}
