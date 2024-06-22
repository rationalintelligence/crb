//! A runtime for composable blocks.

use crate::context::{Context, Label};
use anyhow::Error;
use async_trait::async_trait;
use futures::stream::AbortHandle;

/// A runtime that can be executed as
/// a standalone activity.
#[async_trait]
pub trait StandaloneRuntime<T> {
    /// Returns a runtime that has to be used in an async context.
    fn new(input: T, label: Label) -> Self;

    /// Run routine in place.
    async fn run(self);
}

/// A runtime that can be executed by a supervisor.
#[async_trait]
pub trait SupervisedRuntime<T>
where
    Self: Send + 'static,
{
    /// Type of the composable block's contenxt.
    type Context: Context;

    /// Used by a lifetime tracker of the supervisor to stop it.
    /// It's the separate type that wraps address made by a runtime.
    fn get_interruptor(&mut self) -> Box<dyn Interruptor>;

    /// Interruptor can interrupt this routine.
    ///
    /// The `notifier` is passed by a reference to fully avoid cloning
    /// or passing it somewhere to let it outlive this trackable object.
    async fn routine(self);

    /// Gets a reference to a context.
    fn context(&self) -> &Self::Context;
}

/// The interruptor used internally by a supervisor
/// context or by a standalone routine.
pub trait Interruptor: Send + 'static {
    /// Interrupte a trackable runtime.
    fn interrupt_trackable(&self) -> Result<(), Error>;
}

impl Interruptor for AbortHandle {
    fn interrupt_trackable(&self) -> Result<(), Error> {
        self.abort();
        Ok(())
    }
}
