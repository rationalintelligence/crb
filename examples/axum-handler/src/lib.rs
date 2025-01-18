use axum::{extract::Request, handler::Handler, response::{Response, IntoResponse}};
use crb::agent::{Agent, RunAgent};
use futures::{Future, FutureExt};
use http::StatusCode;
use std::marker::PhantomData;
use std::pin::Pin;

pub trait AxumAgent: Agent<Context: Default> {
    type Response: IntoResponse;
    fn from_request(request: Request) -> Self;
    fn to_response(self) -> Option<Self::Response>;
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
    A: AxumAgent,
    T: 'static,
    S: 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn call(self, req: Request, _state: S) -> Self::Future {
        FutureExt::boxed(async {
            Runner::<A>::new(req).perform().await
        })
    }
}

struct Runner<A: AxumAgent> {
    runtime: RunAgent<A>,
}

impl<A: AxumAgent> Runner<A> {
    fn new(req: Request) -> Self {
        let agent = A::from_request(req);
        let runtime = RunAgent::new(agent);
        Self {
            runtime,
        }
    }

    async fn perform(&mut self) -> Response {
        let result = {
            if let Err(err) = self.runtime.perform().await {
                Err(err)
            } else if let Some(agent) = self.runtime.agent.take() {
                Ok(agent.to_response())
            } else {
                Ok(None)
            }
        };
        match result {
            Ok(Some(response)) => response.into_response(),
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
}
