use anyhow::Error;
use async_trait::async_trait;
use crb::agent::{Agent, AgentSession, OnEvent, Standalone};

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
    async fn handle(&mut self, event: Print, _ctx: &mut Self::Context) -> Result<(), Error> {
        println!("{}", event.0);
        Ok(())
    }
}

#[tokio::test]
async fn test_agent() -> Result<(), Error> {
    let mut addr = TestAgent.spawn();
    let print = Print("Hello, Agent!".into());
    addr.event(print)?;
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
