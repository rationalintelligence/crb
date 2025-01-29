use anyhow::Result;
use async_trait::async_trait;
use crb_agent::Standalone;
use crb_runtime::{InterruptionLevel, Interruptor};
use tokio::{select, signal};

#[async_trait]
pub trait Main {
    async fn main(self) -> Result<()>;
}

#[async_trait]
impl<A> Main for A
where
    A: Standalone,
    A::Context: Default,
{
    async fn main(self) -> Result<()> {
        let mut address = self.spawn();
        let mut level = InterruptionLevel::EVENT;
        loop {
            select! {
                _ = signal::ctrl_c() => {
                    address.interrupt_with_level(level);
                    level = level.next();
                }
                _ = address.join() => {
                    break;
                }
            }
        }
        Ok(())
    }
}
