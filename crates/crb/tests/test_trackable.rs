use anyhow::Error;
use async_trait::async_trait;
use crb_actor::kit::{Actor, ActorSession, DoActor, OnEvent};
use crb_runtime::task::InteractiveTask;
use crb_supervisor::actor::{Supervisor, SupervisorSession};

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
    let mut addr = DoActor::new(Main).spawn_connected();
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
