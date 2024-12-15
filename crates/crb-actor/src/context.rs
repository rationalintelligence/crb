use crate::message::Envelope;
use crb_core::mpsc;

pub struct ActorContext<T> {
    msg_rx: mpsc::UnboundedReceiver<Envelope<T>>,
}

impl<T> ActorContext<T> {
    pub async fn next_envelope(&mut self) -> Option<Envelope<T>> {
        self.msg_rx.recv().await
    }
}
