use anyhow::Result;
use async_trait::async_trait;
use crb::agent::{
    Address, Agent, AgentSession, Context, Duty, ManagedContext, Next, OnEvent, Standalone,
};
use crb::core::Slot;
use derive_more::From;
use notify::{
    recommended_watcher, Event, EventHandler, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::PathBuf;

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
    debouncer: Option<()>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            path: DEFAULT_PATH.into(),
            watcher: Slot::empty(),
            debouncer: None,
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

    fn interrupt(&mut self, ctx: &mut Self::Context) {
        self.watcher.take().ok();
        ctx.shutdown();
    }
}

struct Configure;

#[async_trait]
impl Duty<Configure> for FileWatcher {
    async fn handle(&mut self, _: Configure, ctx: &mut Self::Context) -> Result<Next<Self>> {
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
    async fn handle(&mut self, result: EventResult, _ctx: &mut Self::Context) -> Result<()> {
        let event = result?;
        println!("{} file changed: {:?}", self.path.display(), event);
        if self.debouncer.is_none() {}
        Ok(())
    }
}
