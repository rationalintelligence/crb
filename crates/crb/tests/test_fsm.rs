use anyhow::{Error, Result};
use async_trait::async_trait;
use crb::agent::{Agent, AgentSession, DoAsync, Next, Standalone};

enum State {
    First,
    Second,
    Third,
}

impl State {
    fn next(&self) -> Next<Task> {
        match self {
            State::First => Next::do_async(First),
            State::Second => Next::do_async(Second),
            State::Third => Next::do_async(Third),
        }
    }
}

struct Task {
    state: State,
}

impl Standalone for Task {}

impl Agent for Task {
    type Context = AgentSession<Self>;
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        self.state.next()
    }
}

struct First;

#[async_trait]
impl DoAsync<First> for Task {
    async fn once(&mut self, _: &mut First) -> Result<Next<Self>> {
        self.state = State::Second;
        Ok(self.state.next())
    }
}

struct Second;

#[async_trait]
impl DoAsync<Second> for Task {
    async fn once(&mut self, _: &mut Second) -> Result<Next<Self>> {
        self.state = State::Third;
        Ok(self.state.next())
    }
}

struct Third;

#[async_trait]
impl DoAsync<Third> for Task {
    async fn once(&mut self, _: &mut Third) -> Result<Next<Self>> {
        Ok(Next::done())
    }
}

#[tokio::test]
async fn test_fsm() -> Result<(), Error> {
    let state = State::First;
    let mut addr = Task { state }.spawn();
    addr.join().await?;
    Ok(())
}
