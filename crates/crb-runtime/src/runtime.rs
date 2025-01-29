//! A runtime for composable blocks.

use crate::context::ReachableContext;
use crate::interruptor::{InterruptionLevel, Interruptor};
use async_trait::async_trait;
use std::ops::DerefMut;

/// A runtime that can be executed by a supervisor.
#[async_trait]
pub trait Runtime: Send + 'static {
    /// Used by a lifetime tracker of the supervisor to stop it.
    /// It's the separate type that wraps address made by a runtime.
    fn get_interruptor(&mut self) -> Box<dyn Interruptor>;

    fn interruption_level(&self) -> InterruptionLevel {
        InterruptionLevel::default()
    }

    async fn routine(&mut self);
}

pub trait InteractiveRuntime: Runtime {
    /// Type of the composable block's contenxt.
    type Context: ReachableContext;

    fn address(&self) -> <Self::Context as ReachableContext>::Address;
}

#[async_trait]
impl Runtime for Box<dyn Runtime> {
    fn get_interruptor(&mut self) -> Box<dyn Interruptor> {
        self.deref_mut().get_interruptor()
    }

    async fn routine(&mut self) {
        self.deref_mut().routine().await
    }
}
