use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crb_agent::{Agent, AgentSession, DoAsync, Next, ToRecipient};
use crb_core::Tag;
use crb_send::{Recipient, Sender};
use futures::{Stream, StreamExt};
use std::pin::Pin;

pub struct Drainer<T> {
    recipient: Option<Recipient<T>>,
    stream: Pin<Box<dyn Stream<Item = T> + Send>>,
}

impl<T> Drainer<T> {
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = T> + Send + 'static,
    {
        Self {
            recipient: None,
            stream: Box::pin(stream),
        }
    }

    pub fn drain_to(&mut self, recipient: impl ToRecipient<T>) {
        self.recipient = Some(recipient.to_recipient());
    }
}

impl<T> Agent for Drainer<T>
where
    T: Tag,
{
    type Context = AgentSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(())
    }
}

#[async_trait]
impl<T> DoAsync for Drainer<T>
where
    T: Tag,
{
    async fn repeat(&mut self, _: &mut ()) -> Result<Option<Next<Self>>> {
        if let Some(recipient) = self.recipient.as_mut() {
            if let Some(item) = self.stream.next().await {
                recipient.send(item)?;
                Ok(None)
            } else {
                Ok(Some(Next::done()))
            }
        } else {
            Ok(Some(Next::fail(anyhow!(
                "Recepient is not set for a drainer."
            ))))
        }
    }
}
