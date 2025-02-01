use anyhow::Result;
use async_trait::async_trait;
use crb::agent::{Agent, AgentSession, Context, Next, OnEvent, Standalone};
use crb::superagent::{Drainer, Item, Supervisor, SupervisorSession};
use futures::stream;

struct TestSupervisor;

impl Standalone for TestSupervisor {}

impl Supervisor for TestSupervisor {
    type GroupBy = ();
}

impl Agent for TestSupervisor {
    type Context = SupervisorSession<Self>;

    fn initialize(&mut self, ctx: &mut Context<Self>) -> Next<Self> {
        ctx.spawn_agent(Child, ());
        ctx.spawn_agent(Child, ());
        ctx.spawn_agent(Child, ());
        ctx.spawn_agent(Child, ());
        ctx.spawn_agent(Child, ());

        let stream = stream::repeat_with(|| ());
        let drainer = Drainer::new(stream);
        ctx.assign(drainer, (), ());

        Next::events()
    }
}

#[async_trait]
impl OnEvent<Item<()>> for TestSupervisor {
    async fn handle(&mut self, _event: Item<()>, _ctx: &mut Context<Self>) -> Result<()> {
        Ok(())
    }
}

struct Child;

impl Agent for Child {
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::done()
    }
}

#[tokio::test]
async fn test_termination() -> Result<()> {
    let mut addr = TestSupervisor.spawn();
    addr.interrupt()?;
    addr.join().await?;
    Ok(())
}
