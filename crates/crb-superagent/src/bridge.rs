use crb_agent::{OnEvent, TheEvent, ToAddress};
use crb_core::{mpsc, sync::Mutex};

pub struct EventBridge<T> {
    tx: mpsc::UnboundedSender<T>,
    rx: Mutex<Option<mpsc::UnboundedReceiver<T>>>,
}

impl<T> EventBridge<T>
where
    T: TheEvent,
{
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

    pub async fn subscribe<A>(&self, addr: impl ToAddress<A>)
    where
        A: OnEvent<T>,
    {
        let rx = self.rx.lock().await.take();
        if let Some(mut rx) = rx {
            let address = addr.to_address();
            // TODO: Use async `Drainer` here?
            crb_core::spawn(async move {
                while let Some(event) = rx.recv().await {
                    address.event(event).ok();
                }
            });
        }
    }
}
