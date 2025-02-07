use anyhow::{anyhow, Result};
use crb_core::mpsc;
use crb_core::time::{sleep_until, Sleep};
use futures::Future;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub struct Interval {
    command_tx: mpsc::UnboundedSender<IntervalCommand>,
    stream: Option<IntervalStream>,
}

impl Interval {
    pub fn set_interval(&self, interval: Duration) -> Result<()> {
        self.command_tx
            .send(IntervalCommand::SetInterval(interval))
            .map_err(|_| anyhow!("Can't set the interval."))
    }

    pub fn events(&mut self) -> Result<IntervalStream> {
        self.stream
            .take()
            .ok_or_else(|| anyhow!("Interval events stream has detached already."))
    }
}

enum IntervalCommand {
    SetInterval(Duration),
}

pub struct IntervalStream {
    current_interval: Duration,
    last_tick: Instant,
    next_deadline: Instant,
    sleep: Pin<Box<Sleep>>,
    command_rx: mpsc::UnboundedReceiver<IntervalCommand>,
}

impl Interval {
    pub fn new() -> Self {
        let initial_interval = Duration::from_secs(1);
        let now = Instant::now();
        let next_deadline = now + initial_interval;
        let sleep = Box::pin(sleep_until(next_deadline.into()));
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let stream = IntervalStream {
            current_interval: initial_interval,
            last_tick: now,
            next_deadline,
            sleep,
            command_rx,
        };
        Interval {
            command_tx,
            stream: Some(stream),
        }
    }
}

impl IntervalStream {
    fn update_deadline(&mut self) {
        let new_deadline = self.last_tick + self.current_interval;
        self.next_deadline = new_deadline;
        self.sleep.as_mut().reset(new_deadline.into());
    }
}

pub struct Tick(pub Instant);

impl Stream for IntervalStream {
    type Item = Tick;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            while let Poll::Ready(cmd) = Pin::new(&mut self.command_rx).poll_recv(cx) {
                if let Some(IntervalCommand::SetInterval(new_interval)) = cmd {
                    self.current_interval = new_interval;
                    self.update_deadline();
                } else {
                    // The handle was closed
                    return Poll::Ready(None);
                }
            }

            let now = Instant::now();
            if now >= self.next_deadline {
                self.last_tick = now;
                self.update_deadline();
                return Poll::Ready(Some(Tick(now)));
            }

            match self.sleep.as_mut().poll(cx) {
                Poll::Ready(_) => continue,
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
