use anyhow::Result;
use axum::{extract::Request, handler::Handler, response::Response};
use crb::agent::{Agent, RunAgent};
use futures::{Future, FutureExt};
use http::StatusCode;
use std::marker::PhantomData;
use std::pin::Pin;

pub trait RequestAgent: Agent<Context: Default, Output = Response> {
    fn from_request(request: Request) -> Self;
}

pub struct AgentHandler<A> {
    _type: PhantomData<A>,
}

unsafe impl<A> Sync for AgentHandler<A> {}

impl<A> Clone for AgentHandler<A> {
    fn clone(&self) -> Self {
        Self { _type: PhantomData }
    }
}

impl<A, T, S> Handler<T, S> for AgentHandler<A>
where
    A: RequestAgent,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(self, req: Request, _state: S) -> Self::Future {
        FutureExt::boxed(async {
            let agent = A::from_request(req);
            let mut runtime = RunAgent::new(agent);
            let result = runtime.perform_and_return().await;
            handle_errors(result)
        })
    }
}

fn handle_errors(res: Result<Option<Response>>) -> Response {
    match res {
        Ok(Some(response)) => response,
        Ok(None) => {
            let mut response = Response::new("Handler has interrupted".into());
            *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
            response
        }
        Err(err) => {
            let mut response = Response::new(err.to_string().into());
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            response
        }
    }
}
