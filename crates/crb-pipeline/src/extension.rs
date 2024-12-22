use crate::{InitialMessage, Pipeline};
use anyhow::Result;
use crb_actor::Address;
use crb_core::types::Clony;

pub trait AddressExt {
    fn ingest<M>(&mut self, message: M) -> Result<()>
    where
        M: Clony;
}

impl AddressExt for Address<Pipeline> {
    fn ingest<M>(&mut self, message: M) -> Result<()>
    where
        M: Clony,
    {
        let message = InitialMessage::new(message);
        self.send(message)
    }
}
