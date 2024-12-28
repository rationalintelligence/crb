pub mod async_task;
pub mod task;

#[cfg(feature = "sync")]
pub mod sync_task;

#[cfg(feature = "sync")]
pub mod hybryd_task;

pub mod kit {
    pub use crate::task::{Task, TaskHandle, JobHandle};
    pub use crate::async_task::{AsyncTask, DoAsync};

    #[cfg(feature = "sync")]
    pub use crate::sync_task::{SyncTask, DoSync};

    #[cfg(feature = "sync")]
    pub use crate::hybryd_task::{HybrydTask, DoHybrid, NextState, Activity, SyncActivity};
}
