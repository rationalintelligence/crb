//! A runtime for composable blocks.

use crate::context::Context;
use crate::interruptor::Interruptor;
use async_trait::async_trait;

/// A runtime that can be executed by a supervisor.
#[async_trait]
pub trait SupervisedRuntime
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

pub trait StandaloneRuntime: SupervisedRuntime + Sized {
    /// Run routine in place.
    fn spawn(self) -> <Self::Context as Context>::Address {
        let address = self.context().address().clone();
        crb_core::spawn(self.routine());
        address
    }
}
