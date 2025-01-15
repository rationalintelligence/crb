use anyhow::Result;
use axum::{extract::Request, response::Response, routing::get, Router};
use crb::agent::{Agent, AgentSession, Next};
use crb_example_axum_handler::{AgentHandler, RequestAgent};
use std::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/", get(AgentHandler::<HelloWorld, (), ()>::new()));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

impl RequestAgent for HelloWorld {
    fn from_request(_request: Request) -> Self {
        println!("HELLO!");
        Self
    }
}

pub struct HelloWorld;

impl Agent for HelloWorld {
    type Context = AgentSession<Self>;
    type Output = Mutex<Response>;

    fn begin(&mut self) -> Next<Self> {
        Next::interrupt()
    }
}
