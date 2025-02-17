pub mod async_performer;
pub mod consume_performer;
pub mod events_performer;
pub mod interrupt_performer;

#[cfg(feature = "sync")]
pub mod sync_performer;

use crate::address::Envelope;
use crate::agent::Agent;
use crate::context::Context;
use anyhow::Error;
use async_trait::async_trait;
use std::fmt;

pub trait AgentState: Send + 'static {}

impl<T> AgentState for T where T: Send + 'static {}

pub struct Next<T: ?Sized> {
    pub(crate) transition: Box<dyn StatePerformer<T>>,
}

impl<T: Agent> Next<T> {
    pub fn new(performer: impl StatePerformer<T>) -> Self {
        Self {
            transition: Box::new(performer),
        }
    }
}

pub enum TransitionCommand<T> {
    Next(Next<T>),
    Stop(StopReason),
    ProcessEvents,
    InContext(Envelope<T>),
}

impl<T> fmt::Debug for TransitionCommand<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Next(_) => "Next(_)",
            Self::Stop(reason) => &format!("Stop({reason:?})"),
            Self::ProcessEvents => "ProcessEvents",
            Self::InContext(_) => "InContext(_)",
        };
        write!(f, "TransitionCommand::{}", value)
    }
}

pub enum StopReason {
    Failed(Error),
    Stopped,
}

impl fmt::Debug for StopReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Failed(_) => "Failed(_)",
            Self::Stopped => "Stopped",
        };
        write!(f, "StopReason::{}", value)
    }
}

pub enum ConsumptionReason {
    Transformed,
    Crashed(Error),
}

impl fmt::Debug for ConsumptionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Transformed => "Transformed",
            Self::Crashed(_) => "Crashed(_)",
        };
        write!(f, "ConsumptionReason::{}", value)
    }
}

pub enum Transition<T: Agent> {
    Continue {
        agent: T,
        command: TransitionCommand<T>,
    },
    Consume {
        reason: ConsumptionReason,
    },
}

impl<T: Agent> fmt::Debug for Transition<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Continue { command, .. } => {
                write!(f, "Transition::{command:?}")
            }
            Self::Consume { reason } => {
                write!(f, "Transition::{reason:?}")
            }
        }
    }
}

#[async_trait]
pub trait StatePerformer<T: Agent>: Send + 'static {
    async fn perform(&mut self, agent: T, session: &mut Context<T>) -> Transition<T>;
}
