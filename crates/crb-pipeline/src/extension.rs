use crate::pipeline::{InitialMessage, Pipeline, PipelineState};
use anyhow::Result;
use crb_agent::Address;

pub trait AddressExt {
    fn ingest<M>(&mut self, message: M) -> Result<()>
    where
        M: Clone + Sync + Send + 'static;
}

impl<State: PipelineState> AddressExt for Address<Pipeline<State>> {
    fn ingest<M>(&mut self, message: M) -> Result<()>
    where
        M: Clone + Sync + Send + 'static,
    {
        let message = InitialMessage::new(message);
        self.send(message)
    }
}
