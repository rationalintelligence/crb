use anyhow::Error;
use async_trait::async_trait;
use crb_agent::{Address, Agent, AgentContext, AgentSession, MessageFor, RunAgent};
use crb_runtime::{InteractiveRuntime, Interruptor, ManagedContext, ReachableContext, Runtime};
use derive_more::{Deref, DerefMut, From, Into};
use std::collections::{BTreeMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use typed_slab::TypedSlab;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, From, Into)]
pub struct ActivityId(usize);

pub trait Supervisor: Agent {
    type GroupBy: Debug + Ord + Clone + Sync + Send + Eq + Hash;

    fn finished(&mut self, _rel: &Relation<Self>, _ctx: &mut Self::Context) {}
}

pub trait SupervisorContext<S: Supervisor> {
    fn session(&mut self) -> &mut SupervisorSession<S>;
}

#[derive(Deref, DerefMut)]
pub struct SupervisorSession<S: Supervisor> {
    #[deref]
    #[deref_mut]
    pub session: AgentSession<S>,
    pub tracker: Tracker<S>,
}

impl<S: Supervisor> Default for SupervisorSession<S> {
    fn default() -> Self {
        Self {
            session: AgentSession::default(),
            tracker: Tracker::new(),
        }
    }
}

impl<S: Supervisor> ReachableContext for SupervisorSession<S> {
    type Address = Address<S>;

    fn address(&self) -> &Self::Address {
        self.session.address()
    }
}

impl<S: Supervisor> AsRef<Address<S>> for SupervisorSession<S> {
    fn as_ref(&self) -> &Address<S> {
        self.address()
    }
}

impl<S: Supervisor> ManagedContext for SupervisorSession<S> {
    fn is_alive(&self) -> bool {
        self.session.is_alive()
    }

    fn shutdown(&mut self) {
        self.tracker.terminate_all();
        if self.tracker.is_terminated() {
            self.session.shutdown();
        }
    }

    fn stop(&mut self) {
        self.session.stop();
    }
}

impl<S: Supervisor> AgentContext<S> for SupervisorSession<S> {
    fn session(&mut self) -> &mut AgentSession<S> {
        &mut self.session
    }
}

impl<S: Supervisor> SupervisorContext<S> for SupervisorSession<S> {
    fn session(&mut self) -> &mut SupervisorSession<S> {
        self
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

pub struct Tracker<S: Supervisor> {
    groups: BTreeMap<S::GroupBy, Group>,
    activities: TypedSlab<ActivityId, Activity<S>>,
    terminating: bool,
}

impl<S: Supervisor> Default for Tracker<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Supervisor> Tracker<S> {
    pub fn new() -> Self {
        Self {
            groups: BTreeMap::new(),
            activities: TypedSlab::new(),
            terminating: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty() && self.activities.is_empty()
    }

    pub fn is_terminated(&self) -> bool {
        self.terminating && self.is_empty()
    }

    pub fn terminate_group(&mut self, group: S::GroupBy) {
        if let Some(group) = self.groups.get(&group) {
            for id in group.ids.iter() {
                if let Some(activity) = self.activities.get_mut(*id) {
                    activity.interrupt();
                }
            }
        }
    }

    pub fn terminate_all(&mut self) {
        self.try_terminate_next();
    }

    fn register_activity(&mut self, group: S::GroupBy, interruptor: Interruptor) -> Relation<S> {
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
        Relation { id, group }
    }

    fn unregister_activity(&mut self, rel: &Relation<S>) {
        if let Some(activity) = self.activities.remove(rel.id) {
            // TODO: check rel.group == activity.group ?
            if let Some(group) = self.groups.get_mut(&activity.group) {
                group.ids.remove(&rel.id);
                if group.ids.is_empty() {
                    self.groups.remove(&activity.group);
                }
            }
        }
        if self.terminating {
            self.try_terminate_next();
        }
    }

    fn existing_groups(&self) -> Vec<S::GroupBy> {
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
}

impl<S> SupervisorSession<S>
where
    S: Supervisor,
    S::Context: SupervisorContext<S>,
{
    pub fn spawn_agent<A>(
        &mut self,
        input: A,
        group: S::GroupBy,
    ) -> (<A::Context as ReachableContext>::Address, Relation<S>)
    where
        A: Agent,
        A::Context: Default,
    {
        let runtime = RunAgent::<A>::new(input);
        self.spawn_runtime(runtime, group)
    }

    pub fn spawn_runtime<B>(
        &mut self,
        trackable: B,
        group: S::GroupBy,
    ) -> (<B::Context as ReachableContext>::Address, Relation<S>)
    where
        B: InteractiveRuntime,
    {
        let addr = trackable.address();
        let rel = self.spawn_trackable(trackable, group);
        (addr, rel)
    }

    pub fn spawn_trackable<B>(&mut self, mut trackable: B, group: S::GroupBy) -> Relation<S>
    where
        B: Runtime,
    {
        let interruptor = trackable.get_interruptor();
        let rel = self.tracker.register_activity(group, interruptor);
        let detacher = DetacherFor {
            supervisor: self.address().clone(),
            rel: rel.clone(),
        };

        let fut = async move {
            trackable.routine().await;
            // This notification equals calling `detach_trackable`
            if let Err(err) = detacher.detach() {
                log::error!("Can't notify a supervisor to detach an activity: {err}");
            }
        };
        crb_core::spawn(fut);
        rel
    }
}

struct Activity<S: Supervisor> {
    group: S::GroupBy,
    // TODO: Consider to use JobHandle here
    interruptor: Interruptor,
}

impl<S: Supervisor> Activity<S> {
    fn interrupt(&mut self) {
        self.interruptor.stop(false);
    }
}

pub struct Relation<S: Supervisor> {
    pub id: ActivityId,
    pub group: S::GroupBy,
}

impl<S: Supervisor> Clone for Relation<S> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            group: self.group.clone(),
        }
    }
}

struct DetachFrom<S: Supervisor> {
    rel: Relation<S>,
}

#[async_trait]
impl<S> MessageFor<S> for DetachFrom<S>
where
    S: Supervisor,
    S::Context: SupervisorContext<S>,
{
    async fn handle(self: Box<Self>, agent: &mut S, ctx: &mut S::Context) -> Result<(), Error> {
        let session = SupervisorContext::session(ctx);
        session.tracker.unregister_activity(&self.rel);
        if session.tracker.is_terminated() {
            session.session.shutdown();
        }
        agent.finished(&self.rel, ctx);
        Ok(())
    }
}

pub struct DetacherFor<S: Supervisor> {
    rel: Relation<S>,
    supervisor: <S::Context as ReachableContext>::Address,
}

impl<S> DetacherFor<S>
where
    S: Supervisor,
    S::Context: SupervisorContext<S>,
{
    pub fn detach(self) -> Result<(), Error> {
        let msg = DetachFrom { rel: self.rel };
        self.supervisor.send(msg)
    }
}
