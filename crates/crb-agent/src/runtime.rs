use crate::agent::Agent;
use crate::context::AgentContext;
use crate::performers::{Transition, TransitionCommand};
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::kit::{
    Context, Failures, InteractiveRuntime, InteractiveTask, Interruptor, Runtime, Task,
};
use futures::stream::Abortable;

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
impl<A: Agent> InteractiveTask<A> for RunAgent<A> {}

#[async_trait]
impl<A: Agent> InteractiveRuntime for RunAgent<A> {
    type Context = A::Context;

    fn address(&self) -> <Self::Context as Context>::Address {
        self.context.address().clone()
    }
}

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
                        Transition::Continue { mut agent, command } => match command {
                            TransitionCommand::Next(Ok(next_state)) => {
                                pair = (agent, Some(next_state));
                            }
                            TransitionCommand::Next(Err(err)) => {
                                let (agent, next_state) =
                                    next_state.transition.fallback(agent, err).await;
                                pair = (agent, Some(next_state));
                            }
                            TransitionCommand::Process => {
                                pair = (agent, None);
                            }
                            TransitionCommand::Interrupted => {
                                pair = (agent, None);
                                break;
                            }
                            TransitionCommand::InContext(envelope) => {
                                envelope
                                    .handle(&mut agent, &mut self.context)
                                    .await
                                    .expect("Agent's loopback should never fail");
                                let next_state = self.context.session().next_state.take();
                                pair = (agent, next_state);
                            }
                        },
                        Transition::Crashed(err) => {
                            return Err(err);
                        }
                    }
                } else {
                    let result = agent.event(&mut self.context).await;
                    self.failures.put(result);
                    let next_state = self.context.session().next_state.take();
                    pair = (agent, next_state);
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
