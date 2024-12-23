use crate::runtime::{Task, TaskRuntime};
use crate::typeless_task::TypelessTask;
use crb_core::JoinHandle;
use crb_runtime::kit::{Entrypoint, Interruptor, Runtime};
use std::marker::PhantomData;

pub struct TypedTask<T> {
    interruptor: Interruptor,
    handle: JoinHandle<()>,
    cancel_on_drop: bool,
    _run: PhantomData<T>,
}

impl<T: Task> TypedTask<T> {
    pub fn spawn(task: T) -> Self {
        let mut runtime = TaskRuntime::new(task);
        let interruptor = runtime.get_interruptor();
        let handle = crb_core::spawn(runtime.entrypoint());
        Self {
            interruptor,
            handle,
            cancel_on_drop: false,
            _run: PhantomData,
        }
    }

    pub fn cancel_on_drop(&mut self, cancel: bool) {
        self.cancel_on_drop = cancel;
    }
}

impl<T> Drop for TypedTask<T> {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.handle.abort();
        }
    }
}

/*
impl<T> From<TypedTask<T>> for TypelessTask {
    fn from(typed: TypedTask<T>) -> Self {
        typed.task
    }
}
*/
