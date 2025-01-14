use anyhow::{Error, Result};
use async_trait::async_trait;
use crb::agent::{Agent, AgentSession, Context, Next, Standalone};
use crb_pipeline::Stage;
use tokio::time::{sleep, Duration};

struct FirstProcessor {
    value: Option<u16>,
}

#[async_trait]
impl Agent for FirstProcessor {
    type Context = AgentSession<Self>;
    type Output = ();

    fn initialize(&mut self, _ctx: &mut Context<Self>) -> Next<Self> {
        println!("FirstProcessor");
        if let Some(value) = self.value.as_mut() {
            *value *= 2;
        }
        Next::done()
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
impl Agent for SecondProcessor {
    type Context = AgentSession<Self>;
    type Output = ();

    fn initialize(&mut self, _ctx: &mut Context<Self>) -> Next<Self> {
        println!("SecondProcessor");
        if let Some(value) = self.value.as_mut() {
            *value *= 2;
        }
        Next::done()
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
    // console_subscriber::init();
    let mut pipeline = Pipeline::new();

    // Routing
    use crb_pipeline::*;
    pipeline.route::<Input<u8, _>, Agent<FirstProcessor>>();
    pipeline.route::<Agent<FirstProcessor>, Agent<SecondProcessor>>();

    // pipeline.route_map::<FirstProcessor, SecondProcessor>();
    // pipeline.route_split::<FirstProcessor, SecondProcessor>();
    // pipeline.route_merge::<FirstProcessor, SecondProcessor>();

    let mut addr = pipeline.spawn();
    addr.ingest(8u8)?;
    sleep(Duration::from_secs(1)).await;
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
