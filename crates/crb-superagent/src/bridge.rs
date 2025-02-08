use anyhow::{anyhow, Result};
use crb_agent::TheEvent;
use crb_core::{mpsc, sync::Mutex};
// TODO: Move to the core
use futures::Stream;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub struct EventBridge<T> {
    tx: mpsc::UnboundedSender<T>,
    rx: Mutex<Option<mpsc::UnboundedReceiver<T>>>,
}

impl<T: TheEvent> Default for EventBridge<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TheEvent> EventBridge<T> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx: Mutex::new(Some(rx)),
        }
    }

    pub fn send(&self, msg: T) {
        self.tx.send(msg).ok();
    }

    pub async fn events(&self) -> Result<impl Stream<Item = T>> {
        self.rx
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow!("Receiver of the EventBridge has consumed already."))
            .map(UnboundedReceiverStream::new)
    }
}
