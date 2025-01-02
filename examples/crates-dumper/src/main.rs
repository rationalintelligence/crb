use anyhow::Result;
use crb::agent::{RunAgent, Task};
use crb_example_crates_dumper::CratesLoader;

#[tokio::main]
async fn main() -> Result<()> {
    RunAgent::new(CratesLoader::new()).run().await;
    Ok(())
}
