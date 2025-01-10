use anyhow::{Error, Result};
use async_trait::async_trait;
use crb::agent::{Agent, AgentSession, Next, OnEvent, Standalone};
use crb::superagent::{Supervisor, SupervisorSession};

struct Printer;

impl Agent for Printer {
    type Context = AgentSession<Self>;
    type Output = ();
}

struct Print(pub String);

#[async_trait]
impl OnEvent<Print> for Printer {
    async fn handle(&mut self, event: Print, _ctx: &mut Self::Context) -> Result<()> {
        println!("{}", event.0);
        Ok(())
    }
}

struct Main;

impl Standalone for Main {}

#[async_trait]
impl Agent for Main {
    type Context = SupervisorSession<Self>;
    type Output = ();

    fn initialize(&mut self, ctx: &mut Self::Context) -> Next<Self> {
        ctx.event(SendPrint)
            .map(|_| Next::events())
            .unwrap_or_else(Next::fail)
    }
}

struct SendPrint;

#[async_trait]
impl OnEvent<SendPrint> for Main {
    async fn handle(&mut self, _event: SendPrint, ctx: &mut Self::Context) -> Result<()> {
        let printer = ctx.spawn_agent(Printer, ());
        let print = Print("Hello, Trackable!".into());
        printer.event(print)?;
        Ok(())
    }
}

impl Supervisor for Main {
    type GroupBy = ();
}

#[tokio::test]
async fn test_trackable() -> Result<(), Error> {
    let mut addr = Main.spawn();
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
