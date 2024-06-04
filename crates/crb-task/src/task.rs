use anyhow::Error;
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
        T: Future<Output = Result<(), Error>> + Send + 'static,
    {
        let handle = crb_core::spawn(async {
            if let Err(err) = fut.await {
                log::error!("The service task has failed: {}", err);
            }
        });
        Self { handle }
    }
}
