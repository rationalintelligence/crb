pub mod agent;
pub mod extension;
pub mod meta;
pub mod pipeline;
pub mod service;
pub mod stage;

pub mod kit {
    pub use crate::agent::{stage::Agent, AgentStage};
    pub use crate::extension::AddressExt;
    pub use crate::pipeline::Pipeline;
    pub use crate::service::{stage::Input, InputStage};
    pub use crate::stage::Stage;
}
