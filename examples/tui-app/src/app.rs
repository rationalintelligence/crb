use crate::events::EventsDrainer;
use crate::state::AppState;
use anyhow::Result;
use async_trait::async_trait;
use crb::agent::{Agent, AgentSession, Context, DoAsync, DoSync, Next, OnEvent, ManagedContext};
use crb::superagent::{Supervisor, SupervisorSession};
use crossterm::event::{Event, KeyCode};
use ratatui::DefaultTerminal;

pub struct TuiApp {
    terminal: Option<DefaultTerminal>,
    state: AppState,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            terminal: None,
            state: AppState::new(),
        }
    }
}

impl Supervisor for TuiApp {
    type BasedOn = AgentSession<Self>;
    type GroupBy = ();
}

impl Agent for TuiApp {
    type Context = SupervisorSession<Self>;

    fn begin(&mut self) -> Next<Self> {
        Next::do_async(Configure)
    }
}

struct Configure;

#[async_trait]
impl DoAsync<Configure> for TuiApp {
    async fn handle(&mut self, _: Configure, ctx: &mut Context<Self>) -> Result<Next<Self>> {
        let terminal = ratatui::try_init()?;
        self.terminal = Some(terminal);
        let drainer = EventsDrainer::new(&ctx);
        ctx.spawn_agent(drainer, ());
        Ok(Next::do_sync(Render))
    }
}

#[async_trait]
impl OnEvent<Event> for TuiApp {
    async fn handle(&mut self, event: Event, ctx: &mut Context<Self>) -> Result<()> {
        let next_state = match event {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => Next::do_async(Terminate),
                _ => {
                    self.state.plus_crab();
                    Next::do_sync(Render)
                }
            },
            _ => Next::do_sync(Render),
        };
        ctx.do_next(next_state);
        Ok(())
    }
}

struct Render;

impl DoSync<Render> for TuiApp {
    fn once(&mut self, _: &mut Render) -> Result<Next<Self>> {
        if let Some(terminal) = self.terminal.as_mut() {
            terminal.draw(|frame| self.state.render(frame))?;
        }
        Ok(Next::events())
    }
}

struct Terminate;

#[async_trait]
impl DoAsync<Terminate> for TuiApp {
    async fn handle(&mut self, _: Terminate, ctx: &mut Context<Self>) -> Result<Next<Self>> {
        self.terminal.take();
        ratatui::try_restore()?;
        ctx.shutdown();
        Ok(Next::events())
    }
}
