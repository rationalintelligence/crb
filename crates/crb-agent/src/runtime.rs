use crate::agent::Agent;
use crate::context::AgentContext;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::kit::{Failures, Interruptor, Runtime, Task};
use futures::stream::Abortable;

pub trait AgentState: Send + 'static {}

impl<T> AgentState for T where T: Send + 'static {}

pub struct Next<T: ?Sized> {
    transition: Box<dyn StatePerformer<T>>,
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

pub enum Transition<T> {
    Next(T, Result<Next<T>>),
    Interrupted(T),
    Process(T),
    Crashed(Error),
}

#[async_trait]
pub trait StatePerformer<T: Agent>: Send + 'static {
    async fn perform(&mut self, agent: T, session: &mut T::Context) -> Transition<T>;
    async fn fallback(&mut self, agent: T, err: Error) -> (T, Next<T>);
}

pub struct RunAgent<T: Agent> {
    pub agent: Option<T>,
    pub context: T::Context,
    pub failures: Failures,
}

impl<T: Agent> RunAgent<T> {
    pub fn new(agent: T) -> Self
    where
        T::Context: Default,
    {
        Self {
            agent: Some(agent),
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
        if let Some(mut agent) = self.agent.take() {
            // let session = self.context.session();

            // Initialize
            let initial_state = agent.initialize(&mut self.context);
            let mut pair = (agent, Some(initial_state));

            // Events or States
            while self.context.session().controller.is_active() {
                let (mut agent, next_state) = pair;
                if let Some(mut next_state) = next_state {
                    let res = next_state
                        .transition
                        .perform(agent, &mut self.context)
                        .await;
                    match res {
                        Transition::Next(agent, Ok(next_state)) => {
                            pair = (agent, Some(next_state));
                        }
                        Transition::Next(agent, Err(err)) => {
                            let (agent, next_state) =
                                next_state.transition.fallback(agent, err).await;
                            pair = (agent, Some(next_state));
                        }
                        Transition::Process(agent) => {
                            pair = (agent, None);
                        }
                        Transition::Crashed(err) => {
                            return Err(err);
                        }
                        Transition::Interrupted(agent) => {
                            pair = (agent, None);
                            break;
                        }
                    }
                } else {
                    let result = agent.event(&mut self.context).await;
                    self.failures.put(result);
                    pair = (agent, self.context.session().next_state.take());
                }
            }

            // Finalize
            let mut agent = pair.0;
            let output = agent.finalize(&mut self.context);
            self.agent = Some(agent);
            Ok(output)
        } else {
            Err(Error::msg("Agent's agent has consumed already."))
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
