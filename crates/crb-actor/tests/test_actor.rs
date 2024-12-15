use anyhow::Error;
use async_trait::async_trait;
use crb_actor::{Actor, ActorContext, Standalone};

struct TestActor;

impl Actor for TestActor {
    type Context = ActorContext<Self>;
    type GroupBy = ();
}

struct Print(pub String);

/*
#[async_trait]
impl OnEvent<Print> for TestActor {
    type Error = Error;
    async fn handle(&mut self, event: Print, _ctx: &mut Self::Context<Self>) -> Result<(), Error> {
        println!("{}", event.0);
        Ok(())
    }
}
*/

#[tokio::test]
async fn test_actor() -> Result<(), Error> {
    let mut addr = TestActor.spawn();
    let print = Print("Hello, World!".into());
    /*
    addr.event(print)?;
    addr.interrupt()?;
    */
    addr.join().await?;
    Ok(())
}
