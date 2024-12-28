use crate::finalizer::{BoxFinalizer, Finalizer};
use crate::routine::Routine;
use async_trait::async_trait;
use crb_core::time::Duration;
use crb_runtime::kit::{
    Context, Controller, Failures, InteractiveRuntime, Interruptor, ManagedContext, Runtime, Task,
};

pub struct RoutineRuntime<R: Routine> {
    pub routine: R,
    pub context: R::Context,
    pub failures: Failures,
}

impl<R: Routine> RoutineRuntime<R> {
    pub fn new(routine: R) -> Self
    where
        R::Context: Default,
    {
        Self {
            routine,
            context: R::Context::default(),
            failures: Failures::default(),
        }
    }
}

#[async_trait]
impl<R: Routine> InteractiveRuntime for RoutineRuntime<R> {
    type Context = R::Context;

    fn address(&self) -> <Self::Context as Context>::Address {
        self.context.address().clone()
    }
}

#[async_trait]
impl<R: Routine> Runtime for RoutineRuntime<R> {
    fn get_interruptor(&mut self) -> Interruptor {
        self.context.session().controller().interruptor.clone()
    }

    async fn routine(&mut self) {
        let ctx = &mut self.context;
        let result = self.routine.routine(ctx).await;
        self.failures.put(result);
    }
}

pub struct RoutineSession<R: Routine> {
    controller: Controller,
    /// Interval between repeatable routine calls
    interval: Duration,
    finalizer: Option<BoxFinalizer<R::Output>>,
}

impl<R: Routine> Default for RoutineSession<R> {
    fn default() -> Self {
        let controller = Controller::default();
        Self {
            controller,
            interval: Duration::from_secs(5),
            finalizer: None,
        }
    }
}

impl<R: Routine> RoutineSession<R> {
    /// Set repeat interval.
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn set_finalizer(&mut self, finalizer: impl Finalizer<R::Output>) {
        self.finalizer = Some(Box::new(finalizer));
    }

    pub fn take_finalizer(&mut self) -> Option<BoxFinalizer<R::Output>> {
        self.finalizer.take()
    }
}

impl<R: Routine> Context for RoutineSession<R> {
    // TODO: TaskAddress that uses a controller internally
    type Address = ();

    fn address(&self) -> &Self::Address {
        &()
    }
}

impl<R: Routine> ManagedContext for RoutineSession<R> {
    fn controller(&mut self) -> &mut Controller {
        &mut self.controller
    }

    fn shutdown(&mut self) {
        self.controller.stop(false).ok();
    }
}

pub trait RoutineContext<R: Routine>: Context {
    fn session(&mut self) -> &mut RoutineSession<R>;
}

impl<R: Routine> RoutineContext<R> for RoutineSession<R> {
    fn session(&mut self) -> &mut RoutineSession<R> {
        self
    }
}

pub trait Standalone: Routine {
    fn spawn(self)
    where
        Self::Context: Default;
}

impl<R> Standalone for R
where
    R: Routine + 'static,
    RoutineRuntime<R>: Task<R>,
{
    fn spawn(self)
    where
        Self::Context: Default,
    {
        let mut runtime = RoutineRuntime::new(self);
        let address = runtime.context.session().address().clone();
        runtime.spawn();
        address
    }
}
