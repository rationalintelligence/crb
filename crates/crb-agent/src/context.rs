use crate::agent::Agent;
use crate::runtime::NextState;
use crb_runtime::kit::{Controller, Context};

pub trait AgentContext<T>: Context {
    fn session(&mut self) -> &mut AgentSession<T>;
}

pub struct AgentSession<T: ?Sized> {
    pub controller: Controller,
    pub next_state: Option<NextState<T>>,
}

impl<T> Default for AgentSession<T> {
    fn default() -> Self {
        Self {
            controller: Controller::default(),
            next_state: None,
        }
    }
}

impl<T> Context for AgentSession<T> {
    type Address = ();

    fn address(&self) -> &Self::Address {
        &()
    }
}

impl<T: Agent> AgentContext<T> for AgentSession<T> {
    fn session(&mut self) -> &mut AgentSession<T> {
        self
    }
}

