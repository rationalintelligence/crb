use anyhow::Error;
use crb::agent::{Agent, Next, Task};
use crb::superagent::{MoltAgent, MoltTo, MoltingSession, NextExt};

#[derive(Debug)]
struct Crab;

struct ShellOne {
    crab: Crab,
    value_1: Option<u8>,
}

impl ShellOne {
    fn new() -> Self {
        Self {
            crab: Crab,
            value_1: None,
        }
    }
}

impl Agent for ShellOne {
    type Context = MoltingSession<Self>;
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        self.value_1 = Some(1);
        Next::molt::<ShellTwo>()
    }
}

impl MoltTo<ShellTwo> for ShellOne {
    fn molt(self) -> Option<ShellTwo> {
        let value_1 = self.value_1?;
        Some(ShellTwo {
            crab: self.crab,
            value_1,
            value_2: None,
        })
    }
}

struct ShellTwo {
    crab: Crab,
    value_1: u8,
    value_2: Option<u16>,
}

impl Agent for ShellTwo {
    type Context = MoltingSession<Self>;
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        self.value_2 = Some(2);
        Next::molt::<ShellThree>()
    }
}

impl MoltTo<ShellThree> for ShellTwo {
    fn molt(self) -> Option<ShellThree> {
        let value_2 = self.value_2?;
        Some(ShellThree {
            crab: self.crab,
            value_1: self.value_1,
            value_2,
        })
    }
}

struct ShellThree {
    crab: Crab,
    value_1: u8,
    value_2: u16,
}

impl Agent for ShellThree {
    type Context = MoltingSession<Self>;
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        println!("Crab = {:?}", self.crab);
        println!("Value 1 = {}", self.value_1);
        println!("Value 2 = {}", self.value_2);
        Next::done()
    }
}

#[tokio::test]
async fn test_molting() -> Result<(), Error> {
    let agent = ShellOne::new();
    MoltAgent::new(agent).run().await;
    Ok(())
}
