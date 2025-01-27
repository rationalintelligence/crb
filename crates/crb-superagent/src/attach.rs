use crb_agent::{Address, Agent};
use crb_runtime::{Runtime, Task, TaskHandle};

pub trait ForwardTo<A: Agent> {
    type Runtime: Runtime;

    fn into_trackable(self, address: Address<A>) -> Self::Runtime;

    fn forward_to(self, address: Address<A>) -> TaskHandle
    where
        Self: Sized,
        Self::Runtime: Task,
    {
        self.into_trackable(address).spawn()
    }
}
