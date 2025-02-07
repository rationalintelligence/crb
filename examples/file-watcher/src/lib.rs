use anyhow::Result;
use async_trait::async_trait;
use crb::agent::{
    Address, Agent, Context, DoAsync, ManagedContext, Next, OnEvent, Standalone,
    ToAddress,
};
use crb::core::{time::Duration, Slot};
use crb::superagent::{Timer, StreamSession, Timeout};
use notify::{
    recommended_watcher, Event, EventHandler, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::PathBuf;

const DEBOUNCE_MS: u64 = 100;

const DEFAULT_PATH: &str = "Cargo.toml";

pub struct FileWatcher {
    path: PathBuf,
    watcher: Slot<RecommendedWatcher>,
    debouncer: Timer,
    counter: usize,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            path: DEFAULT_PATH.into(),
            watcher: Slot::empty(),
            debouncer: Timer::new(),
            counter: 0,
        }
    }
}

impl Standalone for FileWatcher {}

impl Agent for FileWatcher {
    type Context = StreamSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(Initialize)
    }

    fn interrupt(&mut self, ctx: &mut Context<Self>) {
        self.watcher.take().ok();
        self.debouncer.cancel().ok();
        ctx.shutdown();
    }
}

struct Initialize;

#[async_trait]
impl DoAsync<Initialize> for FileWatcher {
    async fn handle(&mut self, _: Initialize, ctx: &mut Context<Self>) -> Result<Next<Self>> {
        ctx.consume(self.debouncer.events()?);
        let forwarder = EventsForwarder::new(ctx);
        let mut watcher = recommended_watcher(forwarder)?;
        watcher.watch(&self.path, RecursiveMode::NonRecursive)?;
        self.watcher.fill(watcher)?;
        Ok(Next::events())
    }
}

struct EventsForwarder {
    address: Address<FileWatcher>,
}

impl EventsForwarder {
    fn new(addr: impl ToAddress<FileWatcher>) -> Self {
        Self {
            address: addr.to_address(),
        }
    }
}

impl EventHandler for EventsForwarder {
    fn handle_event(&mut self, event: EventResult) {
        self.address.event(event).ok();
    }
}

type EventResult = Result<Event, notify::Error>;

#[async_trait]
impl OnEvent<EventResult> for FileWatcher {
    async fn handle(&mut self, result: EventResult, _ctx: &mut Context<Self>) -> Result<()> {
        let _event = result?;
        self.counter += 1;
        let duration = Duration::from_millis(DEBOUNCE_MS);
        self.debouncer.schedule(duration)?;
        Ok(())
    }
}

#[async_trait]
impl OnEvent<Timeout> for FileWatcher {
    async fn handle(&mut self, _: Timeout, _ctx: &mut Context<Self>) -> Result<()> {
        self.debouncer.cancel()?;
        print!("{} file changed.", self.path.display());
        println!(" Debounced events: {}", self.counter);
        self.counter = 0;
        Ok(())
    }
}
