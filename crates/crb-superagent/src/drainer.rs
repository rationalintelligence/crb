use crate::routine::{AsyncRoutine, Routine, SyncRoutine};
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::RunAgent;
use crb_runtime::{JobHandle, Task};
use crb_send::{Recipient, Sender};

pub struct Drainer {
    #[allow(unused)]
    job: Option<JobHandle>,
}

impl Drainer {
    pub fn new_async<R>(routine: R) -> Self
    where
        R: AsyncRoutine,
    {
        Self::spawn(Routine::new_async(routine))
    }

    pub fn new_sync<R>(routine: R) -> Self
    where
        R: SyncRoutine,
    {
        Self::spawn(Routine::new_sync(routine))
    }

    pub fn spawn(routine: Routine) -> Self {
        let mut job = RunAgent::new(routine).spawn().job();
        job.cancel_on_drop(true);
        Self { job: Some(job) }
    }
}

struct AsyncDrainer {}

#[async_trait]
impl AsyncRoutine for AsyncDrainer {
    async fn routine(&mut self) -> Result<()> {
        Ok(())
    }
}

pub trait SyncDrainerFn<T = ()>: FnMut() -> T + Send + 'static {}

impl<F, T> SyncDrainerFn<T> for F where F: FnMut() -> T + Send + 'static {}

struct SyncDrainer<M> {
    func: Box<dyn SyncDrainerFn<Result<M>>>,
    recipient: Recipient<M>,
}

#[async_trait]
impl<M> AsyncRoutine for SyncDrainer<M>
where
    M: Send + 'static,
{
    async fn routine(&mut self) -> Result<()> {
        let message = (self.func)()?;
        self.recipient.send(message)?;
        Ok(())
    }
}
