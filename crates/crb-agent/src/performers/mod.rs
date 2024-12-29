pub mod async_performer;
pub mod interrupt_performer;
pub mod process_performer;

#[cfg(feature = "sync")]
pub mod sync_performer;


use anyhow::{Result, Error};
use crate::agent::Agent;
use async_trait::async_trait;

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
    // TODO: Add
    // Event(Envelope<T>),
}

pub enum Transition<T> {
    Continue {
        agent: T,
        command: TransitionCommand<T>,
    },
    Crashed(Error),
}

#[async_trait]
pub trait StatePerformer<T: Agent>: Send + 'static {
    async fn perform(&mut self, agent: T, session: &mut T::Context) -> Transition<T>;
    async fn fallback(&mut self, agent: T, err: Error) -> (T, Next<T>);
}
