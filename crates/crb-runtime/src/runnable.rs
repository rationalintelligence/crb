//! A runtime for composable blocks.

use crate::context::Context;
use crate::interruptor::Interruptor;
use async_trait::async_trait;

/// A runtime that can be executed by a supervisor.
#[async_trait]
pub trait Runnable: Send + 'static {
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

    // async fn entrypoint(self, context: Self::Context);

    /// Gets a reference to a context.
    fn context(&self) -> &Self::Context;
}

pub trait Standalone: Runnable + Sized {
    fn spawn(self) -> <Self::Context as Context>::Address {
        let address = self.context().address().clone();
        crb_core::spawn(self.routine());
        address
    }
}
