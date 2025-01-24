use anyhow::Result;
use async_trait::async_trait;
use crb::agent::{
    Address, Agent, AgentSession, Context, EventExt, OnEvent, Standalone, UniAddress,
};
use crb::superagent::{InteractExt, OnRequest, Request};

struct TestAgent;

impl Standalone for TestAgent {}

impl Agent for TestAgent {
    type Context = AgentSession<Self>;
}

pub struct TestEvent;

#[async_trait]
impl OnEvent<TestEvent> for TestAgent {
    async fn handle(&mut self, _event: TestEvent, _ctx: &mut Context<Self>) -> Result<()> {
        Ok(())
    }
}

pub struct TestReq;

impl Request for TestReq {
    type Response = ();
}

#[async_trait]
impl OnRequest<TestReq> for TestAgent {
    async fn on_request(&mut self, _req: TestReq, _ctx: &mut Context<Self>) -> Result<()> {
        Ok(())
    }
}

pub trait TestLink
where
    Self: EventExt<TestEvent>,
    Self: InteractExt<TestReq>,
{
}

impl TestLink for Address<TestAgent>
where
    Self: EventExt<TestEvent>,
    Self: InteractExt<TestReq>,
{
}

#[tokio::test]
async fn test_uni_address() -> Result<()> {
    let mut addr = TestAgent.spawn();
    let uni: UniAddress<dyn TestLink> = UniAddress::new(addr.clone());
    uni.event(TestEvent)?;
    uni.interact(TestReq).await?;
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
