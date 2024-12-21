//! A runtime for composable blocks.

use crate::context::Context;
use crate::interruptor::Interruptor;
use async_trait::async_trait;
use std::ops::DerefMut;

/// A runtime that can be executed by a supervisor.
#[async_trait]
pub trait Runtime: Send + 'static {
    /// Used by a lifetime tracker of the supervisor to stop it.
    /// It's the separate type that wraps address made by a runtime.
    fn get_interruptor(&mut self) -> Interruptor;

    async fn routine(&mut self);
}

pub trait OpenRuntime: Runtime {
    /// Type of the composable block's contenxt.
    type Context: Context;

    fn address(&self) -> <Self::Context as Context>::Address;
}

#[async_trait]
pub trait Entrypoint: Runtime + Sized {
    async fn entrypoint(mut self) {
        self.routine().await;
    }
}

impl<T> Entrypoint for T where T: Runtime + Sized {}

#[async_trait]
impl Runtime for Box<dyn Runtime> {
    fn get_interruptor(&mut self) -> Interruptor {
        self.deref_mut().get_interruptor()
    }

    async fn routine(&mut self) {
        self.deref_mut().routine().await
    }
}
