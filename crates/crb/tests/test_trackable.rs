use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_agent::kit::{Agent, AgentSession, Next, OnEvent, Standalone};
use crb_supervisor::agent::{Supervisor, SupervisorSession};

struct Printer;

impl Agent for Printer {
    type Context = AgentSession<Self>;
    type Output = ();
}

struct Print(pub String);

#[async_trait]
impl OnEvent<Print> for Printer {
    type Error = Error;
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
            .map(|_| Next::process())
            .unwrap_or_else(Next::fail)
    }
}

struct SendPrint;

#[async_trait]
impl OnEvent<SendPrint> for Main {
    type Error = Error;

    async fn handle(
        &mut self,
        _event: SendPrint,
        ctx: &mut Self::Context,
    ) -> Result<(), Self::Error> {
        let printer = ctx.spawn_actor(Printer, ());
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
    let res: () = addr.join().await?;
    Ok(())
}
