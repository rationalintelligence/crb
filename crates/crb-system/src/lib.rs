use anyhow::Result;
use async_trait::async_trait;
use crb_agent::Standalone;
use crb_runtime::{InteractiveTask, Runtime};
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
        let mut rt = self.runtime();
        let interruptor = rt.get_interruptor();
        let mut address = rt.spawn_connected();
        let mut level = 0;
        loop {
            select! {
                _ = signal::ctrl_c() => {
                    interruptor.interrupt_with_level(level);
                    level += 1;
                }
                _ = address.join() => {
                    break;
                }
            }
        }
        Ok(())
    }
}
