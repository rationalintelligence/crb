use crate::pipeline::{InitialMessage, Pipeline, PipelineState};
use anyhow::Result;
use crb_actor::kit::Address;
use crb_core::types::Clony;

pub trait AddressExt {
    fn ingest<M>(&mut self, message: M) -> Result<()>
    where
        M: Clony;
}

impl<State: PipelineState> AddressExt for Address<Pipeline<State>> {
    fn ingest<M>(&mut self, message: M) -> Result<()>
    where
        M: Clony,
    {
        let message = InitialMessage::new(message);
        self.send(message)
    }
}
