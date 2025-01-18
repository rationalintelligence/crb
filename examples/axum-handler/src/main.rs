use anyhow::Result;
use async_trait::async_trait;
use axum::{extract::Request, routing::get, Router};
use crb::agent::{Agent, AgentSession, Next, Context};
use crb::superagent::Mission;
use crb_example_axum_handler::{AgentHandler, RequestAgent};

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
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        Next::interrupt()
    }
}

#[async_trait]
impl Mission for HelloWorld {
    type Goal = &'static str;

    async fn deliver(self, _ctx: &mut Context<Self>) -> Option<Self::Goal> {
        Some("Hello!")
    }
}
