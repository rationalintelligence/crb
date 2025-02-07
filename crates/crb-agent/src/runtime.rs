use crate::agent::Agent;
use crate::context::{AgentContext, Context};
use crate::performers::{ConsumptionReason, StopReason, Transition, TransitionCommand};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_runtime::{
    InteractiveRuntime, InteractiveTask, InterruptionLevel, Interruptor, ManagedContext,
    ReachableContext, Runtime, Task,
};
use futures::{stream::Abortable, FutureExt};
use std::future::{Future, IntoFuture};
use std::pin::Pin;

pub struct RunAgent<A: Agent> {
    pub agent: Option<A>,
    pub context: Context<A>,
    pub level: InterruptionLevel,
}

impl<A: Agent> RunAgent<A> {
    pub fn new(agent: A) -> Self
    where
        A::Context: Default,
    {
        Self {
            agent: Some(agent),
            context: Context::wrap(A::Context::default()),
            level: InterruptionLevel::FLAG,
        }
    }

    pub async fn operate(mut self) {
        self.perform_and_report().await;
    }

    pub fn report(&mut self, interrupted: bool) {
        self.context.session().joint.report(interrupted).ok();
    }

    pub async fn perform_and_report(&mut self) {
        self.perform().await;
        let interrupted = self.agent.is_none();
        self.report(interrupted);
    }

    pub async fn perform(&mut self) {
        let name = std::any::type_name::<A>();
        log::info!("Agent {name} started.");
        let result = self.perform_abortable_task().await;
        if let Err(err) = result {
            A::rollback(self.agent.as_mut(), err, &mut self.context).await;
        }
        log::info!("Agent {name} finished.");
    }

    pub async fn perform_abortable_task(&mut self) -> Result<()> {
        let reg = self.context.session().controller.take_registration()?;
        let fut = self.perform_task();
        Abortable::new(fut, reg).await??;
        Ok(())
    }

    async fn perform_task(&mut self) -> Result<()> {
        if let Some(mut agent) = self.agent.take() {
            // let session = self.context.session();

            // Initialize
            let initial_state = agent.initialize(&mut self.context);
            let mut pair = (agent, Some(initial_state));

            // Events or States
            while self.context.is_alive() {
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
                                        agent.failed(err, &mut self.context);
                                    }
                                    StopReason::Stopped => {}
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
                                return Ok(());
                            }
                            ConsumptionReason::Crashed(err) => {
                                return Err(err);
                            }
                        },
                    }
                } else {
                    let result = agent.event(&mut self.context).await;
                    if let Err(err) = result {
                        agent.failed(err, &mut self.context);
                    }
                    let next_state = self.context.session().next_state.take();
                    pair = (agent, next_state);
                }
            }

            // Finalize
            let mut agent = pair.0;
            agent.finalize(&mut self.context);
            self.agent = Some(agent);
            Ok(())
        } else {
            Err(anyhow!("Agent's agent has consumed already."))
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
    fn get_interruptor(&mut self) -> Box<dyn Interruptor> {
        let session = self.context.session();
        let address = session.address().clone();
        Box::new(address)
    }

    fn interruption_level(&self) -> InterruptionLevel {
        self.level
    }

    async fn routine(&mut self) {
        self.perform_and_report().await;
    }
}

#[async_trait]
impl<A: Agent> InteractiveRuntime for RunAgent<A> {
    type Context = A::Context;

    fn address(&self) -> <Self::Context as ReachableContext>::Address {
        self.context.address().clone()
    }
}

impl<A: Agent> IntoFuture for RunAgent<A> {
    type Output = ();
    type IntoFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        self.operate().boxed()
    }
}
