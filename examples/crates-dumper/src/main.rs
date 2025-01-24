use anyhow::Result;
use crb::agent::Runnable;
use crb_example_crates_dumper::CratesLoader;

#[tokio::main]
async fn main() -> Result<()> {
    CratesLoader::new().run().await;
    Ok(())
}
