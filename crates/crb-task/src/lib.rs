pub mod async_task;

#[cfg(feature = "sync")]
pub mod sync_task;

pub mod kit {
    pub use crate::async_task::{Task, TaskRuntime, TypedTask, TypelessTask};

    #[cfg(feature = "sync")]
    pub use crate::sync_task::{SyncTask, SyncTaskRuntime, TypedSyncTask, TypelessSyncTask};
}
