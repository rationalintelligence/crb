use anyhow::Result;
use axum::{extract::Request, routing::get, Router, response::Redirect};
use crb::agent::{Agent, AgentSession, Next};
use crb_example_axum_handler::{AgentHandler, AxumAgent};

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/", get(AgentHandler::<CrbWorld, (), ()>::new()));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

const URL: &str = "https://runtime-blocks.github.io/website/repo/crb/assets/crb-header.png";

pub struct CrbWorld;

impl Agent for CrbWorld {
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::done()
    }
}

impl AxumAgent for CrbWorld {
    type Response = Redirect;

    fn from_request(_request: Request) -> Self {
        Self
    }

    fn to_response(self) -> Option<Self::Response> {
        Some(Redirect::temporary(URL))
    }
}
