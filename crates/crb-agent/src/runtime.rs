use crate::agent::Agent;
use crate::context::AgentContext;
use crate::finalizer::FinalizerFor;
use crate::performers::{ConsumptionReason, StopReason, Transition, TransitionCommand};
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::{
    Context, Failures, InteractiveRuntime, InteractiveTask, Interruptor, Runtime, Task,
};
use futures::stream::Abortable;

pub struct RunAgent<A: Agent> {
    pub agent: Option<A>,
    pub context: A::Context,
    pub failures: Failures,
    pub finalizers: Vec<Box<dyn FinalizerFor<A>>>,
}

impl<A: Agent> RunAgent<A> {
    pub fn new(agent: A) -> Self
    where
        A::Context: Default,
    {
        Self {
            agent: Some(agent),
            context: A::Context::default(),
            failures: Failures::default(),
            finalizers: Vec::new(),
        }
    }
}

impl<T: Agent> RunAgent<T> {
    pub(crate) async fn perform_routine(&mut self) -> Result<()> {
        let reg = self.context.session().controller.take_registration()?;
        let fut = self.perform_task();
        let output = Abortable::new(fut, reg).await??;
        if let Some(output) = output.as_ref() {
            for finalizer in &mut self.finalizers {
                let res = finalizer.finalize(output);
                self.failures.put(res);
            }
        }
        self.context.session().joint.report(output)?;
        Ok(())
    }

    async fn perform_task(&mut self) -> Result<Option<T::Output>> {
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
                            TransitionCommand::Next(next_state) => {
                                pair = (agent, Some(next_state));
                            }
                            TransitionCommand::ProcessEvents => {
                                pair = (agent, None);
                            }
                            TransitionCommand::Stop(reason) => {
                                match reason {
                                    StopReason::Failed(err) => {
                                        agent.failed(&err, &mut self.context);
                                    }
                                    StopReason::Interrupted | StopReason::Done => {}
                                }
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
                        Transition::Consume { reason } => match reason {
                            ConsumptionReason::Transformed(output) => {
                                return Ok(output);
                            }
                            ConsumptionReason::Crashed(err) => {
                                return Err(err);
                            }
                        },
                    }
                } else {
                    let result = agent.event(&mut self.context).await;
                    if let Err(err) = &result {
                        agent.failed(err, &mut self.context);
                    }
                    self.failures.put(result);
                    let next_state = self.context.session().next_state.take();
                    pair = (agent, next_state);
                }
            }

            // Finalize
            let agent = pair.0;
            let output = agent.finalize(&mut self.context);
            Ok(output)
        } else {
            Err(Error::msg("Agent's agent has consumed already."))
        }
    }
}

impl<A: Agent> Task<A> for RunAgent<A> {}
impl<A: Agent> InteractiveTask<A> for RunAgent<A> {}

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
        self.failures.put(result.map(drop));
    }
}

#[async_trait]
impl<A: Agent> InteractiveRuntime for RunAgent<A> {
    type Context = A::Context;

    fn address(&self) -> <Self::Context as Context>::Address {
        self.context.address().clone()
    }
}
