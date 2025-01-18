use crate::agent::Agent;
use crate::context::{AgentContext, Context};
use crate::finalizer::FinalizerFor;
use crate::performers::{ConsumptionReason, StopReason, Transition, TransitionCommand};
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_runtime::{
    Failures, InteractiveRuntime, InteractiveTask, Interruptor, ManagedContext, ReachableContext,
    Runtime, Task,
};
use futures::stream::Abortable;

pub struct RunAgent<A: Agent> {
    pub agent: Option<A>,
    pub context: Context<A>,
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
            context: Context::wrap(A::Context::default()),
            failures: Failures::default(),
            finalizers: Vec::new(),
        }
    }
}

impl<A: Agent> RunAgent<A> {
    pub async fn perform_and_report(&mut self) -> Result<()> {
        self.perform().await?;
        let interrupted = self.agent.is_none();
        self.context.session().joint.report(interrupted)?;
        Ok(())
    }

    pub async fn perform(&mut self) -> Result<()> {
        let reg = self.context.session().controller.take_registration()?;
        let fut = self.perform_task();
        Abortable::new(fut, reg).await??;
        Ok(())
    }

    async fn perform_task(&mut self) -> Result<Option<A::Output>> {
        if let Some(mut agent) = self.agent.take() {
            // let session = self.context.session();

            // Initialize
            let initial_state = agent.initialize(&mut self.context);
            let mut pair = (agent, Some(initial_state));

            // Events or States
            while self.context.session().is_alive() {
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
                            ConsumptionReason::Transformed => {
                                return Ok(None);
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
impl<A> Runtime for RunAgent<A>
where
    A: Agent,
{
    fn get_interruptor(&mut self) -> Interruptor {
        self.context.session().controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        let result = self.perform_and_report().await;
        self.failures.put(result);
    }
}

#[async_trait]
impl<A: Agent> InteractiveRuntime for RunAgent<A> {
    type Context = A::Context;

    fn address(&self) -> <Self::Context as ReachableContext>::Address {
        self.context.address().clone()
    }
}
