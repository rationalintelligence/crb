use crate::Actor;
use crb_runtime::context::ManagedContext;

pub struct ActorRuntime<T: Actor> {
    actor: T,
    context: T::Context,
}

impl<T: Actor> ActorRuntime<T> {
    pub async fn entrypoint(mut self) {
        if let Err(err) = self.actor.initialize(&mut self.context).await {
            log::error!("Initialization of the actor failed: {err}");
        }
        while self.context.controller().is_active() {
            if let Err(err) = self.actor.event(&mut self.context).await {
                log::error!("Event handling for the actor failed: {err}");
            }
        }
        if let Err(err) = self.actor.finalize(&mut self.context).await {
            log::error!("Finalization of the actor failed: {err}");
        }
        if let Err(err) = self.context.status_tx.send(ActorStatus::Done) {
            log::error!("Can't change the status of the terminated actor: {err}");
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ActorStatus {
    Active,
    Done,
}

impl ActorStatus {
    pub fn is_done(&self) -> bool {
        *self == Self::Done
    }
}
