use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_actor::{Actor, ActorSession, Standalone};
use crb_pipeline::{AddressExt, ConductedActor, Pipeline};
use crb_runtime::kit::ManagedContext;
use tokio::time::{sleep, Duration};

struct FirstProcessor {
    value: u16,
}

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
    type Input = u8;
    type Output = u16;

    fn input(input: Self::Input) -> Self {
        Self {
            value: input as u16 * 2,
        }
    }

    fn output(&mut self) -> Self::Output {
        self.value
    }
}

struct SecondProcessor {
    value: u32,
}

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
    type Input = u16;
    type Output = u32;

    fn input(input: Self::Input) -> Self {
        Self {
            value: input as u32 * 2,
        }
    }

    fn output(&mut self) -> Self::Output {
        self.value
    }
}

#[tokio::test]
async fn test_pipeline() -> Result<(), Error> {
    let mut pipeline = Pipeline::new();

    // Routing
    pipeline.input::<u8, FirstProcessor>();
    pipeline.route::<FirstProcessor, SecondProcessor>();

    // pipeline.route_map::<FirstProcessor, SecondProcessor>();
    // pipeline.route_split::<FirstProcessor, SecondProcessor>();
    // pipeline.route_merge::<FirstProcessor, SecondProcessor>();

    let mut addr = pipeline.spawn();
    addr.ingest(8)?;
    sleep(Duration::from_millis(10)).await;
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
