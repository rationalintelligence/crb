use anyhow::Result;
use async_trait::async_trait;
use crb::agent::{Address, Agent, AgentSession, Context, Equip, OnEvent, Standalone};
use derive_more::{Deref, DerefMut, From};

struct TestAgent;

impl Standalone for TestAgent {}

impl Agent for TestAgent {
    type Context = AgentSession<Self>;
    type Output = ();
}

struct Print(pub String);

#[async_trait]
impl OnEvent<Print> for TestAgent {
    async fn handle(&mut self, event: Print, _ctx: &mut Context<Self>) -> Result<()> {
        println!("{}", event.0);
        Ok(())
    }
}

#[derive(Deref, DerefMut, From)]
struct Printer {
    address: Address<TestAgent>,
}

impl Printer {
    fn print(&self, msg: &str) -> Result<()> {
        let print = Print(msg.into());
        self.address.event(print)?;
        Ok(())
    }
}

#[tokio::test]
async fn test_agent() -> Result<()> {
    let mut addr: Printer = TestAgent.spawn().equip();
    addr.print("Hello, Agent!")?;
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
