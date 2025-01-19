use crate::routine::{AsyncRoutine, Routine, SyncRoutine};
use crb_agent::RunAgent;
use crb_runtime::{JobHandle, Task};

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
