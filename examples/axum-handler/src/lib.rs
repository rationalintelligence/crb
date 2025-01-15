use anyhow::Result;
use axum::{extract::Request, handler::Handler, response::Response};
use crb::agent::{Agent, RunAgent};
use futures::{Future, FutureExt};
use http::StatusCode;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Mutex;

pub trait RequestAgent: Agent<Context: Default, Output = Mutex<Response>> {
    fn from_request(request: Request) -> Self;
}

pub struct AgentHandler<A, T, S> {
    _a: PhantomData<A>,
    _t: PhantomData<T>,
    _s: PhantomData<S>,
}

impl<A, T, S> AgentHandler<A, T, S> {
    pub fn new() -> Self {
        Self {
            _a: PhantomData,
            _t: PhantomData,
            _s: PhantomData,
        }
    }
}

unsafe impl<A, T, S> Send for AgentHandler<A, T, S> {}

unsafe impl<A, T, S> Sync for AgentHandler<A, T, S> {}

impl<A, T, S> Clone for AgentHandler<A, T, S> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<A, T, S> Handler<T, S> for AgentHandler<A, T, S>
where
    A: RequestAgent,
    T: 'static,
    S: 'static,
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

fn handle_errors(res: Result<Option<Mutex<Response>>>) -> Response {
    match res {
        Ok(Some(response)) => response.into_inner().unwrap(),
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
