use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, DoAsync, DoSync, Next};

#[async_trait]
pub trait AsyncRoutine: Send + 'static {
    async fn routine(&mut self) -> Result<()>;
}

pub trait SyncRoutine: Send + 'static {
    fn routine(&mut self) -> Result<()>;
}

pub enum Routine {
    AsyncRoutine(Box<dyn AsyncRoutine>),
    SyncRoutine(Box<dyn SyncRoutine>),
    Detached,
}

impl Routine {
    pub fn new_async<R: AsyncRoutine>(routine: R) -> Self {
        Self::AsyncRoutine(Box::new(routine))
    }

    pub fn new_sync<R: SyncRoutine>(routine: R) -> Self {
        Self::SyncRoutine(Box::new(routine))
    }
}

impl Agent for Routine {
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        let mut detached = Self::Detached;
        std::mem::swap(self, &mut detached);
        match detached {
            Self::AsyncRoutine(routine) => Next::do_async(routine),
            Self::SyncRoutine(routine) => Next::do_sync(routine),
            Self::Detached => Next::fail(anyhow!("Detached routine")),
        }
    }
}

#[async_trait]
impl DoAsync<Box<dyn AsyncRoutine>> for Routine {
    async fn repeat(&mut self, boxed: &mut Box<dyn AsyncRoutine>) -> Result<Option<Next<Self>>> {
        boxed.routine().await?;
        Ok(None)
    }
}

#[async_trait]
impl DoSync<Box<dyn SyncRoutine>> for Routine {
    fn repeat(&mut self, boxed: &mut Box<dyn SyncRoutine>) -> Result<Option<Next<Self>>> {
        boxed.routine()?;
        Ok(None)
    }
}
