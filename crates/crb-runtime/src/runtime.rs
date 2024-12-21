//! A runtime for composable blocks.

use crate::context::Context;
use crate::error::Failures;
use crate::interruptor::Interruptor;
use async_trait::async_trait;

/// A runtime that can be executed by a supervisor.
#[async_trait]
pub trait Runtime: Sized + Send + 'static {
    /// Used by a lifetime tracker of the supervisor to stop it.
    /// It's the separate type that wraps address made by a runtime.
    fn get_interruptor(&mut self) -> Interruptor;

    async fn routine(&mut self);

    async fn entrypoint(mut self) {
        self.routine().await;
    }
}

pub trait OpenRuntime: Runtime {
    /// Type of the composable block's contenxt.
    type Context: Context;

    fn address(&self) -> <Self::Context as Context>::Address;
}
