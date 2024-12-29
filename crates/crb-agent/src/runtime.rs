use crate::agent::Agent;
use crate::context::AgentContext;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::kit::{Failures, Interruptor, Runtime, Task};
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
    Interrupted(T),
    Process(T),
    Crashed(Error),
}

#[async_trait]
pub trait StatePerformer<T: Agent>: Send + 'static {
    async fn perform(&mut self, task: T, session: &mut T::Context) -> Transition<T>;
    async fn fallback(&mut self, task: T, err: Error) -> (T, NextState<T>);
}

pub struct RunAgent<T: Agent> {
    pub task: Option<T>,
    pub context: T::Context,
    pub failures: Failures,
}

impl<T: Agent> RunAgent<T> {
    pub fn new(task: T) -> Self
    where
        T::Context: Default,
    {
        Self {
            task: Some(task),
            context: T::Context::default(),
            failures: Failures::default(),
        }
    }
}

impl<T: Agent> Task<T> for RunAgent<T> {}

impl<T: Agent> RunAgent<T> {
    async fn perform_routine(&mut self) -> Result<(), Error> {
        let reg = self.context.session().controller.take_registration()?;
        let fut = self.perform_task();
        let output = Abortable::new(fut, reg).await??;
        // TODO: Distribute outputs
        // TODO: Call finalizers to deliver the result
        // TODO: The default finalizer is = oneshot address self channel!!!!!
        self.context.session().joint.report(output)?;
        Ok(())
    }

    async fn perform_task(&mut self) -> Result<T::Output, Error> {
        if let Some(mut task) = self.task.take() {
            // let session = self.context.session();

            // Initialize
            let initial_state = task.initialize(&mut self.context);
            let mut pair = (task, Some(initial_state));

            // Events or States
            while self.context.session().controller.is_active() {
                let (mut task, next_state) = pair;
                if let Some(mut next_state) = next_state {
                    let res = next_state.transition.perform(task, &mut self.context).await;
                    match res {
                        Transition::Next(task, Ok(next_state)) => {
                            pair = (task, Some(next_state));
                        }
                        Transition::Next(task, Err(err)) => {
                            let (task, next_state) = next_state.transition.fallback(task, err).await;
                            pair = (task, Some(next_state));
                        }
                        Transition::Process(task) => {
                            pair = (task, None);
                        }
                        Transition::Crashed(err) => {
                            return Err(err);
                        }
                        Transition::Interrupted(task) => {
                            pair = (task, None);
                            break;
                        }
                    }
                } else {
                    let result = task.event(&mut self.context).await;
                    self.failures.put(result);
                    pair = (task, self.context.session().next_state.take());
                }
            }

            // Finalize
            let task = pair.0;
            let output = task.finalize(&mut self.context);
            Ok(output)
        } else {
            Err(Error::msg("Agent's task has consumed already."))
        }
    }
}

#[async_trait]
impl<T> Runtime for RunAgent<T>
where
    T: Agent,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.context.session().controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        let result = self.perform_routine().await;
        self.failures.put(result);
    }
}
