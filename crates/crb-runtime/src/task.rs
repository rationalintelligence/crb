use crate::context::ReachableContext;
use crate::interruptor::Interruptor;
use crate::runtime::{InteractiveRuntime, Runtime};
use async_trait::async_trait;
use crb_core::JoinHandle;
use derive_more::{Deref, DerefMut};
use std::marker::PhantomData;

pub trait InteractiveTask<T>: Task<T> + InteractiveRuntime {
    fn spawn_connected(self) -> <Self::Context as ReachableContext>::Address {
        let address = self.address();
        self.spawn();
        address
    }
}

#[async_trait]
pub trait Task<T = ()>: Runtime + Sized {
    fn spawn(mut self) -> TaskHandle<T> {
        let interruptor = self.get_interruptor();
        let handle = crb_core::spawn(async move {
            self.routine().await;
        });
        let job = JobHandle {
            interruptor,
            handle,
            cancel_on_drop: false,
        };
        TaskHandle {
            job,
            _task: PhantomData,
        }
    }

    async fn run(mut self) {
        self.routine().await;
    }
}

#[derive(Deref, DerefMut)]
pub struct TaskHandle<T = ()> {
    #[deref]
    #[deref_mut]
    job: JobHandle,
    _task: PhantomData<T>,
}

impl<T> TaskHandle<T> {
    pub fn job(self) -> JobHandle {
        self.into()
    }
}

impl<T> From<TaskHandle<T>> for JobHandle {
    fn from(task_handle: TaskHandle<T>) -> Self {
        task_handle.job
    }
}

pub struct JobHandle {
    interruptor: Interruptor,
    handle: JoinHandle<()>,
    cancel_on_drop: bool,
}

impl JobHandle {
    pub fn cancel_on_drop(&mut self, cancel: bool) {
        self.cancel_on_drop = cancel;
    }

    pub fn interrupt(&mut self) {
        self.interruptor.stop(true);
    }
}

impl Drop for JobHandle {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.handle.abort();
        }
    }
}
