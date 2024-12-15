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
}

#[async_trait]
impl<T> SupervisedRuntime for RoutineRuntime<T>
where
    T: Routine,
{
    type Context = T::Context;

    fn get_interruptor(&mut self) -> Box<dyn Interruptor> {
        self.context.session().controller.interruptor()
    }

    async fn routine(mut self) {
        let mut ctx = self.context;
        // log::info!(target: ctx.label(), "Task started");
        let result = self.routine.routine(&mut ctx).await;
        if let Err(err) = self.routine.finalize(result).await {
            // log::error!(target: ctx.label(), "Finalize of the task failed: {}", err);
        }
        // log::info!(target: ctx.label(), "Task finished");
    }

    fn context(&self) -> &Self::Context {
        &self.context
    }
}

pub struct RoutineSession {
    controller: Controller,
    /// Interval between repeatable routine calls
    interval: Duration,
}

impl RoutineSession {
    /// Set repeat interval.
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}

impl Context for RoutineSession {
    // TODO: TaskAddress that uses a controller internally
    type Address = ();

    fn address(&self) -> &Self::Address {
        &()
    }
}

impl ManagedContext for RoutineSession {
    fn controller(&mut self) -> &mut Controller {
        &mut self.controller
    }

    fn shutdown(&mut self) {
        self.controller.stop(false).ok();
    }
}

pub trait RoutineContext: Context {
    fn session(&mut self) -> &mut RoutineSession;
}

impl RoutineContext for RoutineSession {
    fn session(&mut self) -> &mut RoutineSession {
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
        crb_core::spawn(runtime.routine());
        address
    }
}
