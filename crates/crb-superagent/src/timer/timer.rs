use anyhow::{anyhow, Result};
use crb_core::mpsc;
use crb_core::time::{sleep_until, Sleep};
use futures::{Future, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Timeout {
    pub scheduled_at: Instant,
}

#[derive(Debug)]
pub enum ScheduleCommand {
    Schedule { delay: Duration },
}

struct PendingEvent {
    baseline: Instant,
    delay: Duration,
}

pub struct TimerStream {
    command_rx: mpsc::UnboundedReceiver<ScheduleCommand>,
    pending: Option<PendingEvent>,
    sleep: Option<Pin<Box<Sleep>>>,
}

pub struct Timer {
    command_tx: mpsc::UnboundedSender<ScheduleCommand>,
    stream: Option<TimerStream>,
}

impl Timer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let stream = TimerStream {
            command_rx: rx,
            pending: None,
            sleep: None,
        };
        Self {
            command_tx: tx,
            stream: Some(stream),
        }
    }

    pub fn schedule(&self, delay: Duration) -> Result<()> {
        self.command_tx
            .send(ScheduleCommand::Schedule { delay })
            .map_err(|_| anyhow!("Can't schedule the task."))
    }

    pub fn events(&mut self) -> Result<TimerStream> {
        self.stream
            .take()
            .ok_or_else(|| anyhow!("Timer events stream has detached already."))
    }
}

impl TimerStream {
    fn timeout(&mut self, scheduled_at: Instant) -> Poll<Option<Timeout>> {
        let event = Timeout { scheduled_at };
        self.pending = None;
        self.sleep = None;
        Poll::Ready(Some(event))
    }
}

impl Stream for TimerStream {
    type Item = Timeout;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            while let Poll::Ready(cmd) = Pin::new(&mut self.command_rx).poll_recv(cx) {
                if let Some(ScheduleCommand::Schedule { delay }) = cmd {
                    let now = Instant::now();
                    if let Some(pending) = &mut self.pending {
                        pending.delay = delay;
                    } else {
                        self.pending = Some(PendingEvent {
                            baseline: now,
                            delay,
                        });
                    }

                    if let Some(pending) = &self.pending {
                        let scheduled_at = pending.baseline + pending.delay;
                        if scheduled_at <= now {
                            return self.timeout(scheduled_at);
                        } else {
                            if let Some(sleep) = &mut self.sleep {
                                sleep.as_mut().reset(scheduled_at.into());
                            } else {
                                self.sleep = Some(Box::pin(sleep_until(scheduled_at.into())));
                            }
                        }
                    }
                } else {
                    // The handle was closed
                    return Poll::Ready(None);
                }
            }

            if let Some(pending) = &self.pending {
                let scheduled_at = pending.baseline + pending.delay;
                let now = Instant::now();
                if now >= scheduled_at {
                    return self.timeout(scheduled_at);
                }
            }

            if let Some(sleep) = &mut self.sleep {
                match sleep.as_mut().poll(cx) {
                    Poll::Ready(_) => continue,
                    Poll::Pending => break,
                }
            }
        }

        Poll::Pending
    }
}
