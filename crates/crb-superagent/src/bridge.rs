use anyhow::{anyhow, Result};
use crb_agent::{Envelope, Event, OnEvent, TheEvent};
use crb_core::{mpsc, sync::Mutex};
// TODO: Move to the core
use futures::Stream;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub struct EventBridge<A> {
    tx: mpsc::UnboundedSender<Envelope<A>>,
    rx: Mutex<Option<mpsc::UnboundedReceiver<Envelope<A>>>>,
}

impl<A> Default for EventBridge<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> EventBridge<A> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx: Mutex::new(Some(rx)),
        }
    }

    pub fn send<E>(&self, msg: E)
    where
        A: OnEvent<E>,
        E: TheEvent,
    {
        let event = Event::envelope(msg);
        self.tx.send(event).ok();
    }

    pub async fn events(&self) -> Result<impl Stream<Item = Envelope<A>>> {
        self.rx
            .lock()
            .await
            .take()
            .ok_or_else(|| anyhow!("Receiver of the EventBridge has consumed already."))
            .map(UnboundedReceiverStream::new)
    }
}
