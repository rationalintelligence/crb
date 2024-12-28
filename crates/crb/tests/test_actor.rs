use anyhow::Error;
use async_trait::async_trait;
// TODO: Use self kits here
use crb_actor::kit::{Actor, ActorSession, OnEvent, Standalone};

struct TestActor;

impl Standalone for TestActor {}

impl Actor for TestActor {
    type Context = ActorSession<Self>;
}

struct Print(pub String);

#[async_trait]
impl OnEvent<Print> for TestActor {
    type Error = Error;
    async fn handle(&mut self, event: Print, _ctx: &mut Self::Context) -> Result<(), Error> {
        println!("{}", event.0);
        Ok(())
    }
}

#[tokio::test]
async fn test_actor() -> Result<(), Error> {
    let mut addr = TestActor.spawn();
    let print = Print("Hello, Actor!".into());
    addr.event(print)?;
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
