use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_actor::{Actor, ActorSession, Standalone};
use crb_pipeline::{AddressExt, ConductedActor, Pipeline};
use crb_runtime::ManagedContext;
use tokio::time::{sleep, Duration};

struct FirstProcessor {}

#[async_trait]
impl Actor for FirstProcessor {
    type Context = ActorSession<Self>;

    async fn initialize(&mut self, ctx: &mut Self::Context) -> Result<()> {
        println!("FirstProcessor");
        ctx.shutdown();
        Ok(())
    }
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

#[async_trait]
impl Actor for SecondProcessor {
    type Context = ActorSession<Self>;

    async fn initialize(&mut self, ctx: &mut Self::Context) -> Result<()> {
        println!("SecondProcessor");
        ctx.shutdown();
        Ok(())
    }
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

    // Routing
    pipeline.input::<(), FirstProcessor>();
    pipeline.route::<FirstProcessor, SecondProcessor>();

    // pipeline.route_map::<FirstProcessor, SecondProcessor>();
    // pipeline.route_split::<FirstProcessor, SecondProcessor>();

    let mut addr = pipeline.spawn();
    addr.ingest(())?;
    sleep(Duration::from_millis(10)).await;
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
