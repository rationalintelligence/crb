use anyhow::Result;
use axum::{routing::get, Router};

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
