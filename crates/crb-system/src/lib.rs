use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Address, Agent};
use crb_runtime::{InterruptionLevel, Interruptor};
use tokio::{select, signal};

#[async_trait]
pub trait Main {
    async fn join_or_signal(self) -> Result<()>;
}

#[async_trait]
impl<A> Main for Address<A>
where
    A: Agent,
    A::Context: Default,
{
    async fn join_or_signal(mut self) -> Result<()> {
        let mut level = InterruptionLevel::EVENT;
        loop {
            select! {
                _ = signal::ctrl_c() => {
                    self.interrupt_with_level(level);
                    level = level.next();
                }
                _ = self.join() => {
                    break;
                }
            }
        }
        Ok(())
    }
}
