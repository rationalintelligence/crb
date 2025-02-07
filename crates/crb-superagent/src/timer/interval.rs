use anyhow::{anyhow, Result};
use crb_core::mpsc;
use crb_core::time::{sleep_until, Sleep};
use futures::Future;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub enum ThrottleCommand {
    SetInterval(Duration),
}

pub struct ThrottledStream {
    last_tick: Instant,
    current_interval: Duration,
    next_deadline: Instant,
    sleep: Pin<Box<Sleep>>,
    command_rx: mpsc::UnboundedReceiver<ThrottleCommand>,
}

pub struct ThrottledHandle {
    command_tx: mpsc::UnboundedSender<ThrottleCommand>,
}

impl ThrottledHandle {
    pub fn set_interval(&self, interval: Duration) -> Result<()> {
        self.command_tx
            .send(ThrottleCommand::SetInterval(interval))
            .map_err(|_| anyhow!("Can't set the interval."))
    }
}

impl ThrottledStream {
    pub fn new() -> (Self, ThrottledHandle) {
        let initial_interval = Duration::from_secs(1);
        let now = Instant::now();
        let next_deadline = now + initial_interval;
        let sleep = Box::pin(sleep_until(next_deadline.into()));
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        (
            ThrottledStream {
                last_tick: now,
                current_interval: initial_interval,
                next_deadline,
                sleep,
                command_rx,
            },
            ThrottledHandle { command_tx },
        )
    }
}

impl Stream for ThrottledStream {
    type Item = Instant;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Instant>> {
        loop {
            while let Poll::Ready(cmd) = Pin::new(&mut self.command_rx).poll_recv(cx) {
                if let Some(ThrottleCommand::SetInterval(new_interval)) = cmd {
                    self.current_interval = new_interval;
                    let new_deadline = self.last_tick + self.current_interval;
                    self.next_deadline = new_deadline;
                    self.sleep.as_mut().reset(new_deadline.into());
                } else {
                    break;
                }
            }

            let now = Instant::now();
            if now >= self.next_deadline {
                let tick = now;
                self.last_tick = tick;
                let new_deadline = tick + self.current_interval;
                self.next_deadline = new_deadline;
                self.sleep.as_mut().reset(new_deadline.into());
                return Poll::Ready(Some(tick));
            }

            match self.sleep.as_mut().poll(cx) {
                Poll::Ready(_) => continue,
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
