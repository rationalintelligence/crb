use anyhow::Result;
use crb::agent::Standalone;
use crb_example_file_watcher::FileWatcher;

#[tokio::main]
async fn main() -> Result<()> {
    let mut addr = FileWatcher::new().spawn();
    // TODO: Support Ctrl-C signals
    addr.join().await?;
    Ok(())
}
