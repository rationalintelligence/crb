use crate::Routine;
use async_trait::async_trait;
use crb_core::time::Duration;
use crb_runtime::{Context, Controller, Failures, Interruptor, ManagedContext, Runtime};

struct RoutineRuntime<T: Routine> {
    routine: T,
    context: T::Context,
    failures: Failures,
}

impl<T> RoutineRuntime<T>
where
    T: Routine,
{
    pub fn new(routine: T) -> Self
    where
        T::Context: Default,
    {
        Self {
            routine,
            context: T::Context::default(),
            failures: Failures::default(),
        }
    }
}

#[async_trait]
impl<T> Runtime for RoutineRuntime<T>
where
    T: Routine,
{
    type Context = T::Context;

    fn get_interruptor(&mut self) -> Box<dyn Interruptor> {
        self.context.session().controller.interruptor()
    }

    fn address(&self) -> <Self::Context as Context>::Address {
        self.context.address().clone()
    }

    async fn routine(mut self) -> Failures {
        let mut ctx = self.context;
        let result = self.routine.routine(&mut ctx).await;
        let result = self.routine.finalize(result).await;
        self.failures.put(result);
        self.failures
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
        crb_core::spawn(runtime.entrypoint());
        address
    }
}
