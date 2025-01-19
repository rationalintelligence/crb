use crate::mission::{async_fn::AnyAsyncFut, sync_fn::AnySyncFn, RunMission};
use crb_runtime::{JobHandle, Task};

pub struct Drainer {
    #[allow(unused)]
    job: Option<JobHandle>,
}

impl Drainer {
    pub fn new_async<F>(fut: F) -> Self
    where
        F: AnyAsyncFut,
    {
        let mut job = RunMission::new_async(fut).spawn().job();
        job.cancel_on_drop(true);
        Self { job: Some(job) }
    }

    pub fn new_sync<F>(func: F) -> Self
    where
        F: AnySyncFn,
    {
        let mut job = RunMission::new_sync(func).spawn().job();
        job.cancel_on_drop(true);
        Self { job: Some(job) }
    }
}
