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
        self.value.as_mut().map(|value| *value *= 2);
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Stage for FirstProcessor {
    type State = ();
    type Config = ();
    type Input = u8;
    type Output = u16;

    fn construct(_config: Self::Config, input: Self::Input, _state: &mut Self::State) -> Self {
        Self {
            value: Some(input as u16),
        }
    }

    async fn next_output(&mut self) -> Option<Self::Output> {
        self.value.take()
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
        self.value.as_mut().map(|value| *value *= 2);
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Stage for SecondProcessor {
    type State = ();
    type Config = ();
    type Input = u16;
    type Output = u32;

    fn construct(_config: Self::Config, input: Self::Input, _state: &mut Self::State) -> Self {
        Self {
            value: Some(input as u32),
        }
    }

    async fn next_output(&mut self) -> Option<Self::Output> {
        self.value.take()
    }
}

#[tokio::test]
async fn test_pipeline() -> Result<(), Error> {
    let mut pipeline = Pipeline::new();

    // Routing
    use crb_pipeline::kit::*;
    pipeline.route::<Input<u8, _>, Actor<FirstProcessor>>();
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
