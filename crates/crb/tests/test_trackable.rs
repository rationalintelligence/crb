use anyhow::Error;
use async_trait::async_trait;
use crb_actor::kit::{Actor, ActorSession, OnEvent, Standalone};
use crb_supervisor::{Supervisor, SupervisorSession};

struct Printer;

impl Actor for Printer {
    type Context = ActorSession<Self>;
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

struct Main;

#[async_trait]
impl Actor for Main {
    type Context = SupervisorSession<Self>;

    async fn initialize(&mut self, ctx: &mut Self::Context) -> Result<(), Error> {
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
    addr.join().await?;
    Ok(())
}
