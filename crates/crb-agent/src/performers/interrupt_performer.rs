use crate::agent::Agent;
use crate::context::Context;
use crate::performers::{Next, StatePerformer, StopReason, Transition, TransitionCommand};
use anyhow::Error;
use async_trait::async_trait;
use crb_runtime::ManagedContext;

impl<A> Next<A>
where
    A: Agent,
{
    pub fn done() -> Self {
        Self::new(StopPerformer {
            call_interrupt: false,
            reason: None,
        })
    }

    pub fn interrupt() -> Self {
        Self::new(StopPerformer {
            call_interrupt: true,
            reason: None,
        })
    }

    pub fn stop() -> Self {
        Self::stop_with_reason(StopReason::Stopped)
    }

    pub fn fail(err: Error) -> Self {
        Self::stop_with_reason(StopReason::Failed(err))
    }

    pub fn todo(reason: impl ToString) -> Self {
        let err = Error::msg(reason.to_string());
        Self::stop_with_reason(StopReason::Failed(err))
    }

    pub fn stop_with_reason(reason: StopReason) -> Self {
        Self::new(StopPerformer {
            call_interrupt: false,
            reason: Some(reason),
        })
    }
}

pub struct StopPerformer {
    call_interrupt: bool,
    reason: Option<StopReason>,
}

#[async_trait]
impl<A> StatePerformer<A> for StopPerformer
where
    A: Agent,
{
    async fn perform(&mut self, mut agent: A, ctx: &mut Context<A>) -> Transition<A> {
        if let Some(reason) = self.reason.take() {
            let command = TransitionCommand::Stop(reason);
            Transition::Continue { agent, command }
        } else {
            if self.call_interrupt {
                agent.interrupt(ctx);
            } else {
                ctx.shutdown();
            }
            let command = TransitionCommand::ProcessEvents;
            Transition::Continue { agent, command }
        }
    }
}
