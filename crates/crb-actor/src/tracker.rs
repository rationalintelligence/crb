use crate::message::MessageFor;
use crate::runtime::{ActorContext, ActorRuntime, ActorSession, Address};
use crate::Actor;
use anyhow::Error;
use async_trait::async_trait;
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
    terminating: bool,
}

impl<A: Actor> Tracker<A> {
    pub fn new() -> Self {
        Self {
            groups: BTreeMap::new(),
            activities: TypedSlab::new(),
            terminating: false,
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

    pub fn terminate_all(&mut self) {
        self.try_terminate_next();
    }

    fn register_activity(
        &mut self,
        group: A::GroupBy,
        interruptor: Box<dyn Interruptor>,
    ) -> TrackerRelation<A> {
        let activity = Activity {
            group: group.clone(),
            interruptor,
        };
        let id = self.activities.insert(activity);
        let group_record = self.groups.entry(group.clone()).or_default();
        group_record.ids.insert(id);
        if group_record.interrupted {
            // Interrupt if the group is terminating
            self.activities.get_mut(id).map(Activity::interrupt);
        }
        TrackerRelation { id, group }
    }

    fn unregister_activity(&mut self, rel: &TrackerRelation<A>) {
        if let Some(activity) = self.activities.remove(rel.id) {
            // TODO: check rel.group == activity.group ?
            if let Some(group) = self.groups.get_mut(&activity.group) {
                group.ids.remove(&rel.id);
            }
        }
        if self.terminating {
            self.try_terminate_next();
        }
    }

    fn existing_groups(&self) -> Vec<A::GroupBy> {
        self.groups.keys().rev().cloned().collect()
    }

    fn try_terminate_next(&mut self) {
        self.terminating = true;
        for group_name in self.existing_groups() {
            if let Some(group) = self.groups.get_mut(&group_name) {
                if !group.interrupted {
                    group.interrupted = true;
                    // Send an interruption signal to all active members of the group.
                    for id in group.ids.iter() {
                        if let Some(activity) = self.activities.get_mut(*id) {
                            activity.interrupt();
                        }
                    }
                }
                if !group.is_finished() {
                    break;
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

        self.register_activity(group, interruptor);

        let fut = async move {
            trackable.routine().await;
            // This notification equals calling `detach_trackable`
            // detacher.detach();
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

struct TrackerRelation<A: Actor> {
    id: ActivityId,
    group: A::GroupBy,
}

struct DetachTrackable<A: Actor> {
    rel: TrackerRelation<A>,
}

#[async_trait]
impl<A> MessageFor<A> for DetachTrackable<A>
where
    // TODO: Make it more flexible, to allow using wrappers
    A: Actor<Context = SupervisorSession<A>>,
{
    async fn handle(self: Box<Self>, _actor: &mut A, ctx: &mut A::Context) -> Result<(), Error> {
        ctx.detach_trackable(&self.rel);
        Ok(())
    }
}

impl<A: Actor> SupervisorSession<A> {
    fn detach_trackable(&mut self, rel: &TrackerRelation<A>) {
        self.tracker.unregister_activity(rel);
    }
}
