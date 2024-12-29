use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::kit::{Controller, Failures, Interruptor, Runtime, Task};
use futures::stream::Abortable;

pub trait AgentState: Send + 'static {}

impl<T> AgentState for T where T: Send + 'static {}

pub struct NextState<T: ?Sized> {
    transition: Box<dyn StatePerformer<T>>,
}

impl<T> NextState<T>
where
    T: Agent,
{
    pub(crate) fn new(performer: impl StatePerformer<T>) -> Self {
        Self {
            transition: Box::new(performer),
        }
    }
}

pub enum Transition<T> {
    Next(T, Result<NextState<T>>),
    Crashed(Error),
    Interrupted,
}

#[async_trait]
pub trait StatePerformer<T>: Send + 'static {
    async fn perform(&mut self, task: T, session: &mut AgentSession<T>) -> Transition<T>;
    async fn fallback(&mut self, task: T, err: Error) -> (T, NextState<T>);
}

pub trait Agent: Sized + Send + 'static {
    fn initial_state(&mut self) -> NextState<Self>;
}

pub struct AgentSession<T> {
    pub controller: Controller,
    pub next_state: Option<NextState<T>>,
}

pub struct RunAgent<T> {
    pub task: Option<T>,
    pub session: AgentSession<T>,
    pub failures: Failures,
}

impl<T: Agent> RunAgent<T> {
    pub fn new(task: T) -> Self {
        let session = AgentSession {
            controller: Controller::default(),
            next_state: None,
        };
        Self {
            task: Some(task),
            session,
            failures: Failures::default(),
        }
    }
}

impl<T: Agent> Task<T> for RunAgent<T> {}

impl<T: Agent> RunAgent<T> {
    async fn perform_routine(&mut self) -> Result<(), Error> {
        let reg = self.session.controller.take_registration()?;
        let fut = self.perform_task();
        Abortable::new(fut, reg).await??;
        Ok(())
    }

    async fn perform_task(&mut self) -> Result<(), Error> {
        if let Some(mut task) = self.task.take() {
            let session = &mut self.session;
            let initial_state = task.initial_state();
            let mut pair = (task, initial_state);
            loop {
                let (task, mut next_state) = pair;
                let res = next_state.transition.perform(task, session).await;
                match res {
                    Transition::Next(task, Ok(next_state)) => {
                        pair = (task, next_state);
                    }
                    Transition::Next(task, Err(err)) => {
                        let (task, next_state) = next_state.transition.fallback(task, err).await;
                        pair = (task, next_state);
                    }
                    Transition::Crashed(err) => {
                        return Err(err);
                    }
                    Transition::Interrupted => {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<T> Runtime for RunAgent<T>
where
    T: Agent,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.session.controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        let result = self.perform_routine().await;
        self.failures.put(result);
    }
}
