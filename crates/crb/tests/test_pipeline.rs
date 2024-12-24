use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_actor::kit::{Actor, ActorSession, Standalone};
use crb_pipeline::kit::Stage;
use crb_runtime::kit::ManagedContext;
use tokio::time::{sleep, Duration};

struct FirstProcessor {
    value: Option<u16>,
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

#[async_trait]
impl Stage for FirstProcessor {
    type Config = ();
    type Input = u8;
    type Output = u16;

    fn construct(_config: Self::Config, input: Self::Input) -> Self {
        Self {
            value: Some(input as u16),
        }
    }

    async fn next_output(&mut self) -> Option<Self::Output> {
        self.value.take().map(|value| value * 2)
    }
}

struct SecondProcessor {
    value: Option<u32>,
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

#[async_trait]
impl Stage for SecondProcessor {
    type Config = ();
    type Input = u16;
    type Output = u32;

    fn construct(_config: Self::Config, input: Self::Input) -> Self {
        Self {
            value: Some(input as u32),
        }
    }

    async fn next_output(&mut self) -> Option<Self::Output> {
        self.value.take().map(|value| value * 2)
    }
}

#[tokio::test]
async fn test_pipeline() -> Result<(), Error> {
    let mut pipeline = Pipeline::new();

    // Routing
    use crb_pipeline::kit::*;
    pipeline.route::<Input<u8>, Actor<FirstProcessor>>();
    pipeline.route::<Actor<FirstProcessor>, Actor<SecondProcessor>>();

    // pipeline.route_map::<FirstProcessor, SecondProcessor>();
    // pipeline.route_split::<FirstProcessor, SecondProcessor>();
    // pipeline.route_merge::<FirstProcessor, SecondProcessor>();

    let mut addr = pipeline.spawn();
    addr.ingest(8u8)?;
    sleep(Duration::from_millis(10)).await;
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
