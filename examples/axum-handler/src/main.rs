use anyhow::Result;
use axum::{extract::Request, routing::get, Router};
use crb::agent::{Agent, AgentSession, Next};
use crb_example_axum_handler::{AgentHandler, AxumAgent};

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/", get(AgentHandler::<HelloWorld, (), ()>::new()));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

pub struct HelloWorld;

impl Agent for HelloWorld {
    type Context = AgentSession<Self>;
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        Next::done()
    }
}

impl AxumAgent for HelloWorld {
    type Response = &'static str;

    fn from_request(_request: Request) -> Self {
        Self
    }

    fn to_response(self) -> Option<Self::Response> {
        Some("Hello!")
    }
}
