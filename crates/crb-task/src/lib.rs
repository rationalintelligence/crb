pub mod hybryd_task;
pub mod performers;

// TODO: Remove
pub mod async_task;
#[cfg(feature = "sync")]
pub mod sync_task;

pub mod kit {
    // TODO: Remove
    pub use crate::async_task::{AsyncTask, DoAsync};
    #[cfg(feature = "sync")]
    pub use crate::sync_task::{DoSync, SyncTask};

    pub use crate::hybryd_task::{DoHybrid, HybrydTask, NextState};
    pub use crate::performers::async_performer::AsyncActivity;

    #[cfg(feature = "sync")]
    pub use crate::performers::sync_performer::SyncActivity;
}
