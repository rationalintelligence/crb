pub mod runtime;
pub mod typed_task;
pub mod typeless_task;

pub mod kit {
    pub use crate::runtime::Task;
    pub use crate::typed_task::TypedTask;
    pub use crate::typeless_task::TypelessTask;
}
