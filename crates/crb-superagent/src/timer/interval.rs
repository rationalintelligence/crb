pub struct Interval {
}

/*
use std::pin::Pin;
use std::time::{Duration, Instant};
use futures::Stream;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio::time::{sleep_until, Sleep};

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
    pub fn set_interval(&self, interval: Duration) {
        let _ = self.command_tx.send(ThrottleCommand::SetInterval(interval));
    }
}

impl ThrottledStream {
    pub fn new(initial_interval: Duration) -> (Self, ThrottledHandle) {
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
                    self.next_deadline = self.last_tick + self.current_interval;
                    self.sleep.as_mut().reset(self.next_deadline.into());
                } else {
                    break;
                }
            }

            let now = Instant::now();
            if now >= self.next_deadline {
                let tick = now;
                self.last_tick = tick;
                self.next_deadline = tick + self.current_interval;
                self.sleep.as_mut().reset(self.next_deadline.into());
                return Poll::Ready(Some(tick));
            }

            match self.sleep.as_mut().poll(cx) {
                Poll::Ready(_) => continue,
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
*/
