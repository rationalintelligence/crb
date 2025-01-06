use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, RunAgent};
use crb_runtime::{Controller, Interruptor, Runtime, Task};

pub trait Molt: Agent<Context = MoltingSession<Self>> {
    type Target: Molt;

    fn molt(self) -> Option<Self::Target>;
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
    pub context: AgentSession<A>,
}

impl<A: Molt> Default for MoltingSession<A> {
    fn default() -> Self {
        Self {
            context: AgentSession::default(),
        }
    }
}

pub struct DoMoltingAgent {
    current_runtime: Option<Box<dyn MoltingRuntime>>,
    controller: Controller,
}

impl DoMoltingAgent {
    pub fn new_agent<A>(agent: A) -> Self
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

impl Task for DoMoltingAgent {}

#[async_trait]
impl Runtime for DoMoltingAgent {
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
