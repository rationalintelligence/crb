use anyhow::Error;
use async_trait::async_trait;
use crb_actor::{Actor, OnEvent, Standalone, SupervisorSession};

struct TestActor;

impl Actor for TestActor {
    type Context = SupervisorSession<Self>;
    type GroupBy = ();
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
    let print = Print("Hello, Trackable!".into());
    addr.event(print)?;
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
