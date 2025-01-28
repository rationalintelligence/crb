use anyhow::Result;
use crb_system::Main;
use crb_example_crates_dumper::CratesLoader;

#[tokio::main]
async fn main() -> Result<()> {
    CratesLoader::new().main().await
}
