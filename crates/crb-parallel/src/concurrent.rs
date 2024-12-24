use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Concurrent {
    type Context;
    type Input;
    type Output;

    async fn initialize(&mut self, _ctx: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    async fn map(&mut self, ctx: &mut Self::Context) -> Option<Self::Input>;

    async fn task(input: Self::Input) -> Self::Output;

    async fn reduce(&mut self, output: Self::Output, ctx: &mut Self::Context);

    async fn finalize(&mut self, _ctx: &mut Self::Context) -> Result<()> {
        Ok(())
    }
}
