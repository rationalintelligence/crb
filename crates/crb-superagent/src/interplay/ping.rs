use super::{Fetcher, Interplay};
use anyhow::Result;
use async_trait::async_trait;
use crb_agent::{Address, Agent, Context, MessageFor};
use crb_core::time::Instant;

pub trait PingExt {
    fn ping(&self) -> Fetcher<Pong>;
}

impl<A: Agent> PingExt for Address<A> {
    fn ping(&self) -> Fetcher<Pong> {
        let now = Instant::now();
        let (interplay, fetcher) = Interplay::new_pair(now);
        let msg = Ping { interplay };
        let res = self.send(msg);
        let fetcher = fetcher.grasp(res);
        fetcher
    }
}

impl<A: Agent> PingExt for Context<A> {
    fn ping(&self) -> Fetcher<Pong> {
        self.address().ping()
    }
}

pub struct Ping {
    pub interplay: Interplay<Instant, Pong>,
}

pub struct Pong {
    pub ping: Instant,
    pub pong: Instant,
}

#[async_trait]
impl<A: Agent> MessageFor<A> for Ping {
    async fn handle(self: Box<Self>, _agent: &mut A, _ctx: &mut Context<A>) -> Result<()> {
        let ping = self.interplay.request;
        let pong = Instant::now();
        self.interplay.responder.send(Pong { ping, pong })
    }
}
