use anyhow::{Error, Result};
use async_trait::async_trait;
use crb::agent::{Address, Agent, AgentSession, Equip, Equipment, OnEvent, Standalone};
use derive_more::{Deref, DerefMut};

struct TestAgent;

impl Standalone for TestAgent {}

impl Agent for TestAgent {
    type Context = AgentSession<Self>;
    type Output = ();
}

struct Print(pub String);

#[async_trait]
impl OnEvent<Print> for TestAgent {
    type Error = Error;
    async fn handle(&mut self, event: Print, _ctx: &mut Self::Context) -> Result<()> {
        println!("{}", event.0);
        Ok(())
    }
}

#[derive(Deref, DerefMut)]
struct Printer {
    address: Address<TestAgent>,
}

impl Equipment for Printer {
    type Agent = TestAgent;

    fn from(address: Address<Self::Agent>) -> Self {
        Self { address }
    }
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
