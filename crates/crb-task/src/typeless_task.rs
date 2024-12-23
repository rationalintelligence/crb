use crate::runtime::Task;
use anyhow::{Error, Result};
use async_trait::async_trait;
use crb_core::JoinHandle;
use futures::Future;

pub struct TypelessTask {
    handle: JoinHandle<()>,
}

impl Drop for TypelessTask {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl TypelessTask {
    pub fn spawn<T>(fut: T) -> Self
    where
        T: Future<Output = Result<()>> + Send + 'static,
    {
        let handle = crb_core::spawn(async {
            if let Err(err) = fut.await {
                log::error!("The service task has failed: {}", err);
            }
        });
        Self { handle }
    }
}

struct FnTask<T> {
    fut: Option<T>,
}

#[async_trait]
impl<T> Task for FnTask<T>
where
    T: Future<Output = Result<()>> + Send + 'static,
{
    async fn routine(&mut self) -> Result<()> {
        self.fut
            .take()
            .ok_or_else(|| Error::msg("Future has taken already"))?
            .await
    }
}
