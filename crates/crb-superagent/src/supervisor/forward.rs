use crb_agent::{Address, Agent};
use crb_core::Tag;
use crb_runtime::{Runtime, Task, TaskHandle};

pub trait ForwardTo<A: Agent, T: Tag> {
    type Runtime: Runtime;

    fn into_trackable(self, address: Address<A>, tag: T) -> Self::Runtime;

    fn forward_to(self, address: Address<A>, tag: T) -> TaskHandle
    where
        Self: Sized,
        Self::Runtime: Task,
    {
        self.into_trackable(address, tag).spawn()
    }
}
