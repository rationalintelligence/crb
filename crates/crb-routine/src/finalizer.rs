use crate::TaskError;
use anyhow::Result;
use async_trait::async_trait;

// TODO:
// - Allow to add actor's address as a finalizer
// - Assign own finalizer by the pipeline
// - Keep the finalizer (boxed) in a context

#[async_trait]
pub trait Finalizer<T>: Send {
    fn finalize(&mut self, result: Result<T, TaskError>) -> Result<()> {
        result?;
        Ok(())
    }
}
