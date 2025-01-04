pub mod async_performer;
pub mod interrupt_performer;
pub mod loopback;
pub mod process_performer;

#[cfg(feature = "sync")]
pub mod sync_performer;

use crate::address::Envelope;
use crate::agent::Agent;
use anyhow::{Error, Result};
use async_trait::async_trait;
use std::fmt;

pub trait AgentState: Send + 'static {}

impl<T> AgentState for T where T: Send + 'static {}

pub struct Next<T: ?Sized> {
    pub(crate) transition: Box<dyn StatePerformer<T>>,
}

impl<T> Next<T>
where
    T: Agent,
{
    pub(crate) fn new(performer: impl StatePerformer<T>) -> Self {
        Self {
            transition: Box::new(performer),
        }
    }
}

pub enum TransitionCommand<T> {
    Next(Result<Next<T>>),
    Interrupted,
    Process,
    InContext(Envelope<T>),
}

impl<T> fmt::Debug for TransitionCommand<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Next(_) => "Next(_)",
            Self::Interrupted => "Interrupted",
            Self::Process => "Process",
            Self::InContext(_) => "InContext(_)",
        };
        write!(f, "TransitionCommand::{}", value)
    }
}

pub enum Transition<T> {
    Continue {
        agent: T,
        command: TransitionCommand<T>,
    },
    Crashed(Error),
}

impl<T> fmt::Debug for Transition<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Continue { command, .. } => {
                write!(f, "Transition::{command:?}")
            }
            Self::Crashed(_) => {
                write!(f, "Transition::Crashed")
            }
        }
    }
}

#[async_trait]
pub trait StatePerformer<T: Agent>: Send + 'static {
    async fn perform(&mut self, agent: T, session: &mut T::Context) -> Transition<T>;
    async fn fallback(&mut self, agent: T, err: Error) -> (T, Next<T>) {
        (agent, Next::interrupt(Some(err)))
    }
}
