use crate::Routine;
use async_trait::async_trait;
use crb_core::time::Duration;
use crb_runtime::context::{Context, ManagedContext};
use crb_runtime::interruptor::Controller;
use crb_runtime::interruptor::Interruptor;
use crb_runtime::runtime::SupervisedRuntime;

struct RoutineRuntime<T: Routine> {
    routine: T,
    context: T::Context,
}

#[async_trait]
impl<T> SupervisedRuntime for RoutineRuntime<T>
where
    T: Routine,
{
    type Context = T::Context;

    fn get_interruptor(&mut self) -> Box<dyn Interruptor> {
        self.context.controller.interruptor()
    }

    async fn routine(mut self) {
        self.entrypoint().await;
    }

    fn context(&self) -> &Self::Context {
        &self.context
    }
}

impl<T> RoutineRuntime<T>
where
    T: Routine,
{
    pub fn new(routine: T) -> Self
    where
        T::Context: Default,
    {
        let context = T::Context::default();
        Self { routine, context }
    }

    async fn entrypoint(mut self) {
        let mut ctx = self.context;
        // log::info!(target: ctx.label(), "Task started");
        let result = self.routine.routine(&mut ctx).await;
        if let Err(err) = self.routine.finalize(result).await {
            // log::error!(target: ctx.label(), "Finalize of the task failed: {}", err);
        }
        // log::info!(target: ctx.label(), "Task finished");
    }
}

pub struct TaskSession {
    controller: Controller,
    /// Interval between repeatable routine calls
    interval: Duration,
}

impl TaskSession {
    /// Set repeat interval.
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}

impl Context for TaskSession {
    // TODO: TaskAddress that uses a controller internally
    type Address = ();

    fn address(&self) -> &Self::Address {
        &()
    }
}

impl ManagedContext for TaskSession {
    fn controller(&mut self) -> &mut Controller {
        &mut self.controller
    }

    fn shutdown(&mut self) {
        // self.msg_rx.close();
    }
}

pub trait TaskContext: Context {
    fn session(&mut self) -> &mut TaskSession;
}

impl TaskContext for TaskSession {
    fn session(&mut self) -> &mut TaskSession {
        self
    }
}

pub trait Standalone: Routine {
    fn spawn(self)
    where
        Self::Context: Default;
}

impl<T: Routine + 'static> Standalone for T {
    fn spawn(self)
    where
        Self::Context: Default,
    {
        let mut runtime = RoutineRuntime::new(self);
        let address = runtime.context.session().address().clone();
        crb_core::spawn(runtime.entrypoint());
        address
    }
}
