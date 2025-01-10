use anyhow::Result;
use crb::agent::Runnable;
use crb_example_tui_app::TuiApp;

#[tokio::main]
async fn main() -> Result<()> {
    TuiApp::new().run().await?;
    // Unblocking stdin
    std::process::exit(0);
}
