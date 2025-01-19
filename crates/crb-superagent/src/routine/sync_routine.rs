use crate::mission::{runtime::RunMission, Mission};
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, Context, DoSync, Next};

impl RunMission<SyncRoutine> {
    pub fn new_sync_routine<F: AnySyncRoutine>(routine: F) -> Self {
        let task = SyncRoutine {
            routine: Box::new(routine),
        };
        Self::new(task)
    }
}

pub trait AnySyncRoutine: FnMut() -> Result<()> + Send + 'static {}

impl<F> AnySyncRoutine for F where F: FnMut() -> Result<()> + Send + 'static {}

pub struct SyncRoutine {
    routine: Box<dyn AnySyncRoutine>,
}

impl Agent for SyncRoutine {
    type Context = AgentSession<Self>;

    fn initialize(&mut self, _ctx: &mut Context<Self>) -> Next<Self> {
        Next::do_sync(CallRoutine)
    }
}

#[async_trait]
impl Mission for SyncRoutine {
    type Goal = ();

    async fn deliver(self, _ctx: &mut Context<Self>) -> Option<Self::Goal> {
        None
    }
}

struct CallRoutine;

impl DoSync<CallRoutine> for SyncRoutine {
    fn repeat(&mut self, _state: &mut CallRoutine) -> Result<Option<Next<Self>>> {
        (self.routine)()?;
        Ok(None)
    }
}
