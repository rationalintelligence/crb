use anyhow::Error;
use async_trait::async_trait;
use crb_actor::{Actor, ActorSession, OnEvent, Standalone};
use crb_supervisor::SupervisorSession;

struct Printer;

impl Actor for Printer {
    type Context = ActorSession<Self>;
    type GroupBy = ();
}

struct Print(pub String);

#[async_trait]
impl OnEvent<Print> for Printer {
    type Error = Error;
    async fn handle(&mut self, event: Print, _ctx: &mut Self::Context) -> Result<(), Error> {
        println!("{}", event.0);
        Ok(())
    }
}

struct Supervisor;

#[async_trait]
impl Actor for Supervisor {
    type Context = SupervisorSession<Self>;
    type GroupBy = ();

    async fn initialize(&mut self, ctx: &mut Self::Context) -> Result<(), Error> {
        let printer = ctx.spawn_actor(Printer, ());
        let print = Print("Hello, Trackable!".into());
        printer.event(print)?;
        Ok(())
    }
}

#[tokio::test]
async fn test_actor() -> Result<(), Error> {
    let mut addr = Supervisor.spawn();
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
