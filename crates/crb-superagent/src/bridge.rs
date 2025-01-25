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

    pub fn subscribe<A>(&'static self, addr: impl ToAddress<A>)
    where
        A: OnEvent<T>,
    {
        let address = addr.to_address();
        crb_core::spawn(async move {
            let rx = self.rx.lock().await.take();
            if let Some(mut rx) = rx {
                // TODO: Use async `Drainer` here?
                while let Some(event) = rx.recv().await {
                    address.event(event).ok();
                }
            }
        });
    }
}
