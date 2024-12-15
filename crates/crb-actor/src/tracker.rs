use crate::runtime::{ActorContext, ActorRuntime, ActorSession, Address};
use crate::Actor;
use anyhow::Error;
use crb_runtime::context::{Context, ManagedContext};
use crb_runtime::interruptor::Interruptor;
use derive_more::{From, Into};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use typed_slab::TypedSlab;

pub struct TrackableSession<T: Actor> {
    session: ActorSession<T>,
    tracker: Tracker<T>,
}

impl<T: Actor> Default for TrackableSession<T> {
    fn default() -> Self {
        Self {
            session: ActorSession::default(),
            tracker: Tracker::new(),
        }
    }
}

impl<T: Actor> ActorContext<T> for TrackableSession<T> {
    fn session(&mut self) -> &mut ActorSession<T> {
        &mut self.session
    }
}

impl<S: Actor> TrackableSession<S> {
    pub fn spawn_actor<A>(&mut self, input: A, group: S::GroupBy) -> Address<A>
    where
        A: Actor,
        A::Context: Default,
        S: Actor,
    {
        let runtime = ActorRuntime::<A>::new(input);
        /*
        let addr = self.spawn_trackable(runtime, group);
        addr
        */
        todo!()
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

impl<T: Actor> Tracker<T> {
    pub fn new() -> Self {
        Self {
            groups: BTreeMap::new(),
            activities: TypedSlab::new(),
        }
    }

    pub fn terminate_group(&mut self, group: T::GroupBy) {
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
