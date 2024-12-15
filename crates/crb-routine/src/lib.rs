mod runtime;

use anyhow::Error;
use async_trait::async_trait;
use crb_core::{
    time::{sleep, timeout, Duration},
    watch,
};
use crb_runtime::context::Context;
use futures::{
    future::{select, Either},
    FutureExt,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("task was interrupted")]
    Interrupted,
    #[error("time for task execution elapsed")]
    Timeout,
    #[error("task failed: {0}")]
    Failed(#[from] Error),
}

async fn just_done(mut status: watch::Receiver<Status>) {
    while status.changed().await.is_ok() {
        if *status.borrow() == Status::Stop {
            break;
        }
    }
}

#[async_trait]
pub trait Routine: Sized + Send + 'static {
    type Context: Context + AsMut<TaskContext>;
    type Output: Send;

    async fn routine(&mut self, ctx: &mut Self::Context) -> Result<Self::Output, Error> {
        let receiver = ctx.as_mut().stop_receiver.clone();
        let time_limit = self.time_limit().await;
        let fut = timeout(time_limit, self.interruptable_routine(ctx));
        let fut = Box::pin(fut);
        let interrupt = Box::pin(just_done(receiver).fuse());
        let either = select(interrupt, fut).await;
        let output = match either {
            Either::Left((_done, _rem_fut)) => Err(TaskError::Interrupted),
            Either::Right((output, _rem_fut)) => Ok(output),
        };
        output??
    }

    async fn interruptable_routine(
        &mut self,
        _ctx: &mut Self::Context,
    ) -> Result<Self::Output, Error> {
        // TODO: Use a flag instead of the channel
        loop {
            let routine_result = self.repeatable_routine().await;
            match routine_result {
                Ok(Some(output)) => {
                    break Ok(output);
                }
                Ok(None) => {
                    self.routine_wait(true).await;
                }
                Err(err) => {
                    // TODO: Report about the error
                    self.routine_wait(false).await;
                }
            }
        }
    }

    async fn repeatable_routine(&mut self) -> Result<Option<Self::Output>, Error> {
        Ok(None)
    }

    async fn time_limit(&mut self) -> Option<Duration> {
        None
    }

    async fn routine_wait(&mut self, _succeed: bool) {
        let duration = Duration::from_secs(5);
        sleep(duration).await
    }

    async fn finalize(&mut self, result: Result<Self::Output, TaskError>) -> Result<(), Error> {
        result?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Status {
    Alive,
    Stop,
}

pub struct TaskContext {
    stop_receiver: watch::Receiver<Status>,
}
