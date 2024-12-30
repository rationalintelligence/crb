pub mod address;
pub mod message;
pub mod agent;
pub mod context;
pub mod performers;
pub mod runtime;

pub mod kit {
    pub use crate::address::{Address, MessageFor};
    pub use crate::agent::{Agent, Standalone};
    pub use crate::context::{AgentContext, AgentSession};
    pub use crate::message::event::OnEvent;
    pub use crate::performers::async_performer::DoAsync;
    pub use crate::runtime::RunAgent;
    pub use crate::performers::Next;

    #[cfg(feature = "sync")]
    pub use crate::performers::sync_performer::DoSync;
}
