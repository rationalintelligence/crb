use async_trait::async_trait;
use crb_agent::performers::{ConsumptionReason, Next, StatePerformer, Transition};
use crb_agent::{Address, Agent, AgentContext, AgentSession, RunAgent};
use crb_runtime::{Context, Controller, Interruptor, ManagedContext, Runtime, Task};
use std::marker::PhantomData;

pub trait NextExt<A> {
    fn molt<T>() -> Self
    where
        A: MoltTo<T>,
        T: Agent<Context = MoltingSession<T>>;
}

impl<A> NextExt<A> for Next<A>
where
    A: Agent<Context = MoltingSession<A>>,
{
    fn molt<T>() -> Self
    where
        A: MoltTo<T>,
        T: Agent<Context = MoltingSession<T>>,
    {
        Self::new(MoltPerformer::<T> { _type: PhantomData })
    }
}

pub trait MoltTo<T>: Sized {
    fn molt(self) -> Option<T> {
        None
    }
}

pub struct MoltPerformer<T> {
    _type: PhantomData<T>,
}

#[async_trait]
impl<A, T> StatePerformer<A> for MoltPerformer<T>
where
    A: Agent<Context = MoltingSession<A>>,
    A: MoltTo<T>,
    T: Agent<Context = MoltingSession<T>>,
{
    async fn perform(&mut self, agent: A, session: &mut A::Context) -> Transition<A> {
        let next_agent = agent.molt();
        if let Some(next_agent) = next_agent {
            let next_runtime = RunAgent::new(next_agent);
            session.next_runtime = Some(Box::new(next_runtime));
        }
        let reason = ConsumptionReason::Transformed(None);
        Transition::Consume { reason }
    }
}

pub struct MoltingSession<A: Agent> {
    pub session: AgentSession<A>,
    pub next_runtime: Option<Box<dyn MoltingRuntime>>,
}

impl<A: Agent> Default for MoltingSession<A> {
    fn default() -> Self {
        Self {
            session: AgentSession::default(),
            next_runtime: None,
        }
    }
}

impl<A: Agent> Context for MoltingSession<A> {
    type Address = Address<A>;

    fn address(&self) -> &Self::Address {
        self.session.address()
    }
}

impl<A> ManagedContext for MoltingSession<A>
where
    A: Agent,
{
    fn is_alive(&self) -> bool {
        self.session.is_alive()
    }

    fn shutdown(&mut self) {
        self.session.shutdown();
    }

    fn stop(&mut self) {
        self.session.stop();
    }
}

impl<A: Agent> AgentContext<A> for MoltingSession<A> {
    fn session(&mut self) -> &mut AgentSession<A> {
        &mut self.session
    }
}

pub struct MoltAgent {
    current_runtime: Option<Box<dyn MoltingRuntime>>,
    controller: Controller,
}

impl MoltAgent {
    pub fn new<A>(agent: A) -> Self
    where
        A: Agent<Context = MoltingSession<A>>,
    {
        let runtime = RunAgent::new(agent);
        Self {
            current_runtime: Some(Box::new(runtime)),
            controller: Controller::default(),
        }
    }
}

impl Task for MoltAgent {}

#[async_trait]
impl Runtime for MoltAgent {
    fn get_interruptor(&mut self) -> Interruptor {
        self.controller.interruptor.clone()
    }

    async fn routine(&mut self) {
        while let Some(mut runtime) = self.current_runtime.take() {
            runtime.routine().await;
            let next_runtime = runtime.do_molting();
            self.current_runtime = next_runtime;
        }
    }
}

pub trait MoltingRuntime: Runtime {
    fn do_molting(self: Box<Self>) -> Option<Box<dyn MoltingRuntime>>;
}

impl<A> MoltingRuntime for RunAgent<A>
where
    A: Agent<Context = MoltingSession<A>>,
{
    fn do_molting(mut self: Box<Self>) -> Option<Box<dyn MoltingRuntime>> {
        self.context.next_runtime.take()
    }
}
