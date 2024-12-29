use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::kit::{Controller, Failures, Interruptor, Runtime, Task, Context};
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
    Process(T),
}

pub trait AgentContext<T>: Context {
    fn session(&mut self) -> &mut AgentSession<T>;
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

#[async_trait]
pub trait StatePerformer<T: Agent>: Send + 'static {
    async fn perform(&mut self, task: T, session: &mut T::Context) -> Transition<T>;
    async fn fallback(&mut self, task: T, err: Error) -> (T, NextState<T>);
}

pub trait Agent: Sized + Send + 'static {
    type Output: Default;
    type Context: AgentContext<Self>;

    fn initialize(&mut self, _ctx: &mut Self::Context) -> NextState<Self> {
        NextState::process()
    }


    fn finalize(&mut self, _ctx: &mut Self::Context) -> Self::Output {
        Self::Output::default()
    }

    // TODO: Add finalizers
    // type Output: Default;

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
        Abortable::new(fut, reg).await??;
        Ok(())
    }

    async fn perform_task(&mut self) -> Result<(), Error> {
        if let Some(mut task) = self.task.take() {
            // let session = self.context.session();

            // Initialize
            let initial_state = task.initialize(&mut self.context);
            let mut pair = (task, Some(initial_state));

            // Events or States
            while self.context.session().controller.is_active() {
                let (task, mut next_state) = pair;
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
                        Transition::Interrupted => {
                            break;
                        }
                    }
                } else {
                    // TODO: Actor's events loop here
                    pair = (task, self.context.session().next_state.take());
                }
            }

            // Finalize
            // TODO: Call finalizers to deliver the result
            // TODO: The default finalizer is = oneshot address self channel!!!!!
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
        self.context.session().controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        let result = self.perform_routine().await;
        self.failures.put(result);
    }
}
