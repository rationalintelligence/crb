use crate::runtime::{ActorContext, ActorRuntime, ActorSession, Address};
use crate::Actor;
use anyhow::Error;
use crb_runtime::context::{Context, ManagedContext};
use crb_runtime::interruptor::{Controller, Interruptor};
use crb_runtime::runtime::SupervisedRuntime;
use derive_more::{From, Into};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use typed_slab::TypedSlab;

pub struct SupervisorSession<T: Actor> {
    session: ActorSession<T>,
    tracker: Tracker<T>,
}

impl<T: Actor> Default for SupervisorSession<T> {
    fn default() -> Self {
        Self {
            session: ActorSession::default(),
            tracker: Tracker::new(),
        }
    }
}

impl<T: Actor> Context for SupervisorSession<T> {
    type Address = Address<T>;

    fn address(&self) -> &Self::Address {
        self.session.address()
    }
}

impl<T: Actor> ManagedContext for SupervisorSession<T> {
    fn controller(&self) -> &Controller {
        self.session.controller()
    }

    fn shutdown(&mut self) {
        self.session.shutdown();
    }
}

impl<T: Actor> ActorContext<T> for SupervisorSession<T> {
    fn session(&mut self) -> &mut ActorSession<T> {
        &mut self.session
    }
}

impl<S: Actor> SupervisorSession<S> {
    pub fn spawn_actor<A>(
        &mut self,
        input: A,
        group: S::GroupBy,
    ) -> <A::Context as Context>::Address
    where
        A: Actor,
        A::Context: Default,
        S: Actor,
    {
        let runtime = ActorRuntime::<A>::new(input);
        self.tracker.spawn_trackable(runtime, group)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into)]
pub struct ActivityId(usize);

impl fmt::Display for ActivityId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Default)]
struct Group {
    interrupted: bool,
    ids: HashSet<ActivityId>,
}

impl Group {
    fn is_finished(&self) -> bool {
        self.interrupted && self.ids.is_empty()
    }
}

pub struct Tracker<T: Actor> {
    groups: BTreeMap<T::GroupBy, Group>,
    activities: TypedSlab<ActivityId, Activity<T>>,
}

impl<A: Actor> Tracker<A> {
    pub fn new() -> Self {
        Self {
            groups: BTreeMap::new(),
            activities: TypedSlab::new(),
        }
    }

    pub fn terminate_group(&mut self, group: A::GroupBy) {
        if let Some(group) = self.groups.get(&group) {
            for id in group.ids.iter() {
                if let Some(activity) = self.activities.get_mut(*id) {
                    if let Err(err) = activity.interrupt() {
                        // TODO: Log
                    }
                }
            }
        }
    }

    pub fn spawn_trackable<B>(
        &mut self,
        mut trackable: B,
        group: A::GroupBy,
    ) -> <B::Context as Context>::Address
    where
        B: SupervisedRuntime,
    {
        let interruptor = trackable.get_interruptor();
        let addr = trackable.context().address().clone();
        // TODO: Add to the tracker
        let fut = async move {
            // TODO: How to use it?
            // let label = trackable.context().label();
            // TODO: Use `address` here instead.
            trackable.routine().await;
        };
        crb_core::spawn(fut);
        addr
    }
}

struct Activity<T: Actor> {
    group: T::GroupBy,
    interruptor: Box<dyn Interruptor>,
}

impl<T: Actor> Activity<T> {
    fn interrupt(&mut self) -> Result<(), Error> {
        self.interruptor.stop(false)
    }
}
