pub mod async_task;
pub mod performers;

#[cfg(feature = "sync")]
pub mod sync_task;

#[cfg(feature = "sync")]
pub mod hybryd_task;

pub mod kit {
    pub use crate::async_task::{AsyncTask, DoAsync};

    #[cfg(feature = "sync")]
    pub use crate::sync_task::{DoSync, SyncTask};

    #[cfg(feature = "sync")]
    pub use crate::performers::sync_performer::SyncActivity;

    pub use crate::performers::async_performer::AsyncActivity;

    #[cfg(feature = "sync")]
    pub use crate::hybryd_task::{DoHybrid, HybrydTask, NextState};
}
