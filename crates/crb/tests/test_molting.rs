use anyhow::Error;
use crb::agent::{Agent, Next, Task};
use crb::superagent::{Molt, MoltAgent, MoltingSession};

struct ShellOne {
    value_1: Option<u8>,
}

impl Agent for ShellOne {
    type Context = MoltingSession<Self>;
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        Next::done()
    }
}

impl Molt for ShellOne {
    type Target = ShellTwo;

    fn molt(self) -> Option<Self::Target> {
        let value_1 = self.value_1?;
        Some(ShellTwo { _value_1: value_1 })
    }
}

struct ShellTwo {
    _value_1: u8,
}

impl Agent for ShellTwo {
    type Context = MoltingSession<Self>;
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        Next::done()
    }
}

impl Molt for ShellTwo {
    type Target = Self;
}

#[tokio::test]
async fn test_agent() -> Result<(), Error> {
    let agent = ShellOne { value_1: None };
    MoltAgent::new(agent).run().await;
    Ok(())
}
