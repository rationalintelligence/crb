//! The module with parts that are compatible with both WASM
//! and non-WASM environments.

pub use tokio::sync::{self, mpsc, oneshot, watch};
pub use tokio::{spawn, task::{spawn_local, JoinHandle}};
pub use tokio::time;
