use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::kit::{Controller, Failures, Interruptor, Runtime, Task};
use futures::stream::Abortable;

pub trait HybrydState: Send + 'static {}

impl<T> HybrydState for T where T: Send + 'static {}

pub struct NextState<T: ?Sized> {
    transition: Box<dyn StatePerformer<T>>,
}

impl<T> NextState<T>
where
    T: HybrydTask,
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
    async fn perform(&mut self, task: T, session: &mut HybrydSession) -> Transition<T>;
    async fn fallback(&mut self, task: T, err: Error) -> (T, NextState<T>);
}

pub trait HybrydTask: Sized + Send + 'static {
    fn initial_state(&mut self) -> NextState<Self>;
}

pub struct HybrydSession {
    pub controller: Controller,
}

pub struct DoHybrid<T> {
    pub task: Option<T>,
    pub session: HybrydSession,
    pub failures: Failures,
}

impl<T: HybrydTask> DoHybrid<T> {
    pub fn new(task: T) -> Self {
        let session = HybrydSession {
            controller: Controller::default(),
        };
        Self {
            task: Some(task),
            session,
            failures: Failures::default(),
        }
    }
}

impl<T: HybrydTask> Task<T> for DoHybrid<T> {}

impl<T: HybrydTask> DoHybrid<T> {
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
impl<T> Runtime for DoHybrid<T>
where
    T: HybrydTask,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.session.controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        let result = self.perform_routine().await;
        self.failures.put(result);
    }
}
