use crate::finalizer::{BoxFinalizer, Finalizer};
use crate::Routine;
use anyhow::Error;
use async_trait::async_trait;
use crb_core::time::Duration;
use crb_runtime::kit::{
    Context, Controller, Entrypoint, Failures, Interruptor, ManagedContext, OpenRuntime, Runtime,
};

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
impl<T: Routine> OpenRuntime for RoutineRuntime<T> {
    type Context = T::Context;

    fn address(&self) -> <Self::Context as Context>::Address {
        self.context.address().clone()
    }
}

#[async_trait]
impl<T: Routine> Runtime for RoutineRuntime<T> {
    fn get_interruptor(&mut self) -> Interruptor {
        self.context.session().controller().interruptor.clone()
    }

    async fn routine(&mut self) {
        let ctx = &mut self.context;
        let output = self.routine.routine(ctx).await;
        let result = if let Some(mut finalizer) = self.context.session().finalizer.take() {
            finalizer.finalize(output).await
        } else {
            output.map(drop).map_err(Error::from)
        };
        self.failures.put(result);
    }
}

pub struct RoutineSession<R: Routine> {
    controller: Controller,
    /// Interval between repeatable routine calls
    interval: Duration,
    finalizer: Option<BoxFinalizer<R::Output>>,
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
