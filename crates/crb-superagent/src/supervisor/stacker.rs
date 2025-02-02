use super::{Supervisor, SupervisorContext};
use crb_agent::{Address, Agent, Context, RunAgent};
use crb_runtime::{InteractiveRuntime, Runtime};

pub struct Stacker<S: Supervisor> {
    scheduled: Vec<(Box<dyn Runtime>, S::GroupBy)>,
}

impl<S: Supervisor> Default for Stacker<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Supervisor> Stacker<S> {
    pub fn new() -> Self {
        Self {
            scheduled: Vec::new(),
        }
    }

    pub fn schedule<A>(&mut self, agent: A, group: S::GroupBy) -> Address<A>
    where
        A: Agent,
        A::Context: Default,
    {
        let runtime = RunAgent::<A>::new(agent);
        let addr = runtime.address();
        self.scheduled.push((Box::new(runtime), group));
        addr
    }

    pub fn spawn_scheduled(&mut self, ctx: &mut Context<S>)
    where
        S::Context: SupervisorContext<S>,
    {
        let runtimes: Vec<_> = self.scheduled.drain(..).collect();
        for (runtime, group) in runtimes {
            ctx.session().spawn_trackable(runtime, group);
        }
    }
}
