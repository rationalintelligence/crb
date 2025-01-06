use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, RunAgent, AgentContext, Address};
use crb_runtime::{Controller, Interruptor, Runtime, Task, Context, ManagedContext};

pub trait Molt: Agent<Context = MoltingSession<Self>> {
    type Target: Molt;

    fn molt(self) -> Option<Self::Target> {
        None
    }
}

pub trait MoltingRuntime: Runtime {
    fn do_molting(self: Box<Self>) -> Option<Box<dyn MoltingRuntime>>;
}

impl<A> MoltingRuntime for RunAgent<A>
where
    A: Molt,
{
    fn do_molting(mut self: Box<Self>) -> Option<Box<dyn MoltingRuntime>> {
        let agent = self.agent.take()?;
        let new_agent = agent.molt()?;
        let runtime = RunAgent::new(new_agent);
        Some(Box::new(runtime))
    }
}

pub struct MoltingSession<A: Molt> {
    pub session: AgentSession<A>,
}

impl<A: Molt> Default for MoltingSession<A> {
    fn default() -> Self {
        Self {
            session: AgentSession::default(),
        }
    }
}

impl<A: Molt> Context for MoltingSession<A> {
    type Address = Address<A>;

    fn address(&self) -> &Self::Address {
        self.session.address()
    }
}

impl<A: Molt> ManagedContext for MoltingSession<A> {
    fn controller(&mut self) -> &mut Controller {
        self.session.controller()
    }

    fn shutdown(&mut self) {
        self.session.shutdown();
    }
}

impl<A: Molt> AgentContext<A> for MoltingSession<A> {
    fn session(&mut self) -> &mut AgentSession<A> {
        &mut self.session
    }
}

impl<A: Molt> MoltingSession<A> {
    pub fn molt(&mut self) {
        self.shutdown();
    }
}

pub struct MoltAgent {
    current_runtime: Option<Box<dyn MoltingRuntime>>,
    controller: Controller,
}

impl MoltAgent {
    pub fn new<A>(agent: A) -> Self
    where
        A: Molt,
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
