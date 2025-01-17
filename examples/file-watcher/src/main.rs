use anyhow::Result;
use async_trait::async_trait;
use crb::agent::{
    Address, Agent, AgentSession, Context, Duty, ManagedContext, Next, OnEvent, Standalone,
};
use crb::core::{time::Duration, Slot};
use crb::superagent::Timeout;
use derive_more::From;
use notify::{
    recommended_watcher, Event, EventHandler, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::PathBuf;

const DEBOUNCE_MS: u64 = 100;

#[tokio::main]
async fn main() -> Result<()> {
    let mut addr = FileWatcher::new().spawn();
    // TODO: Support Ctrl-C signals
    addr.join().await?;
    Ok(())
}

const DEFAULT_PATH: &str = "Cargo.toml";

pub struct FileWatcher {
    path: PathBuf,
    watcher: Slot<RecommendedWatcher>,
    debouncer: Slot<Timeout>,
    counter: usize,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            path: DEFAULT_PATH.into(),
            watcher: Slot::empty(),
            debouncer: Slot::empty(),
            counter: 0,
        }
    }
}

impl Standalone for FileWatcher {}

impl Agent for FileWatcher {
    type Context = AgentSession<Self>;
    type Output = ();

    fn begin(&mut self) -> Next<Self> {
        Next::duty(Configure)
    }

    fn interrupt(&mut self, ctx: &mut Context<Self>) {
        self.watcher.take().ok();
        self.debouncer.take().ok();
        ctx.shutdown();
    }
}

struct Configure;

#[async_trait]
impl Duty<Configure> for FileWatcher {
    async fn handle(&mut self, _: Configure, ctx: &mut Context<Self>) -> Result<Next<Self>> {
        let forwarder = EventsForwarder::from(ctx.address().clone());
        let mut watcher = recommended_watcher(forwarder)?;
        watcher.watch(&self.path, RecursiveMode::NonRecursive)?;
        self.watcher.fill(watcher)?;
        Ok(Next::events())
    }
}

#[derive(From)]
struct EventsForwarder {
    address: Address<FileWatcher>,
}

impl EventHandler for EventsForwarder {
    fn handle_event(&mut self, event: EventResult) {
        self.address.event(event).ok();
    }
}

type EventResult = Result<Event, notify::Error>;

#[async_trait]
impl OnEvent<EventResult> for FileWatcher {
    async fn handle(&mut self, result: EventResult, ctx: &mut Context<Self>) -> Result<()> {
        let _event = result?;
        self.counter += 1;
        if self.debouncer.is_empty() {
            let address = ctx.address().clone();
            let duration = Duration::from_millis(DEBOUNCE_MS);
            let timeout = Timeout::new(address, duration, Tick);
            self.debouncer.fill(timeout)?;
        }
        Ok(())
    }
}

struct Tick;

#[async_trait]
impl OnEvent<Tick> for FileWatcher {
    async fn handle(&mut self, _: Tick, _ctx: &mut Context<Self>) -> Result<()> {
        self.debouncer.take()?;
        println!(
            "{} file changed. Debounced events: {}",
            self.path.display(),
            self.counter
        );
        self.counter = 0;
        Ok(())
    }
}
