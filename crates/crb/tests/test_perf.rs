use anyhow::Result;
use async_trait::async_trait;
use crb::agent::{
    Agent, AgentSession, Duty, ManagedContext, Next, OnEvent, ReachableContext, Standalone,
};
use std::time::{Duration, Instant};

struct TestTime {
    from: Instant,
    counter: usize,
}

impl TestTime {
    fn new() -> Self {
        Self {
            from: Instant::now(),
            counter: 0,
        }
    }

    fn reset(&mut self) {
        self.from = Instant::now();
        self.counter = 0;
    }

    fn inc(&mut self) {
        self.counter += 1;
    }

    fn report_and_reset(&mut self, mode: &str) {
        println!("Total in `{mode}`: {}", self.counter);
        self.reset();
    }

    fn is_done(&self) -> bool {
        self.from.elapsed() >= Duration::from_secs(1)
    }
}

impl Standalone for TestTime {}

impl Agent for TestTime {
    type Context = AgentSession<Self>;
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        self.reset();
        Next::duty(SelfCall)
    }
}

struct SelfCall;

#[async_trait]
impl Duty<SelfCall> for TestTime {
    async fn handle(&mut self, _: SelfCall, ctx: &mut Self::Context) -> Result<Next<Self>> {
        if self.is_done() {
            self.report_and_reset("fsm");
            ctx.address().event(SelfCall)?;
            Ok(Next::events())
        } else {
            self.inc();
            Ok(Next::duty(SelfCall))
        }
    }
}

#[async_trait]
impl OnEvent<SelfCall> for TestTime {
    async fn handle(&mut self, _: SelfCall, ctx: &mut Self::Context) -> Result<()> {
        if self.is_done() {
            self.report_and_reset("actor");
            ctx.shutdown();
        } else {
            self.inc();
            ctx.address().event(SelfCall)?;
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_perf() -> Result<()> {
    let mut addr = TestTime::new().spawn();
    addr.join().await?;
    Ok(())
}
