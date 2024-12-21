use anyhow::Error;
use async_trait::async_trait;
use crb_actor::{Actor, ActorSession, OnEvent, Standalone};
use crb_pipeline::{AddressExt, ConductedActor, Pipeline};
use tokio::time::{sleep, Duration};

struct FirstProcessor {}

impl Actor for FirstProcessor {
    type Context = ActorSession<Self>;
}

impl ConductedActor for FirstProcessor {
    type Input = ();
    type Output = ();

    fn input(_input: Self::Input) -> Self {
        Self {}
    }

    fn output(&mut self) -> Self::Output {
        ()
    }
}

struct SecondProcessor {}

impl Actor for SecondProcessor {
    type Context = ActorSession<Self>;
}

impl ConductedActor for SecondProcessor {
    type Input = ();
    type Output = ();

    fn input(_input: Self::Input) -> Self {
        Self {}
    }

    fn output(&mut self) -> Self::Output {
        ()
    }
}

#[tokio::test]
async fn test_pipeline() -> Result<(), Error> {
    let mut pipeline = Pipeline::new();
    /*
    pipeline! {
        () > FirstProcessor,
        FirstProcessor >> SecondProcessor,
        SecondProcessor > mapper > ThirdProcessor,
        ThirdProcessor => ()
    }
    */
    pipeline.input::<(), FirstProcessor>();
    pipeline.route::<FirstProcessor, SecondProcessor>();
    let mut addr = pipeline.spawn();
    addr.ingest(())?;
    sleep(Duration::from_secs(2)).await;
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
