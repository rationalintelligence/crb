pub mod actor;
pub mod extension;
pub mod meta;
pub mod routine;
pub mod service;
pub mod stage;

pub mod kit {
    pub use crate::actor::ActorStage;
    pub use crate::extension::AddressExt;
    pub use crate::service::InputStage;
    pub use crate::stage::Stage;
    pub use crate::Pipeline;
}

use crb_actor::{Actor, Address};
use crb_runtime::kit::{Context, Runtime};
use crb_supervisor::{Supervisor, SupervisorSession};
use meta::{Metadata, Sequencer};
use stage::Stage;
use stage::{StageDestination, StageSource};
use std::any::type_name;
use typedmap::{TypedDashMap, TypedMapKey};

pub struct Pipeline {
    sequencer: Sequencer,
    routes: TypedDashMap,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            sequencer: Sequencer::default(),
            routes: TypedDashMap::default(),
        }
    }

    pub fn stage<FROM, TO>(&mut self, from: FROM, to: TO)
    where
        FROM: StageSource,
        TO: StageDestination,
        TO::Stage: Stage<Input = <FROM::Stage as Stage>::Output>,
    {
        let key = from.source();
        let generator = to.destination();
        self.routes.entry(key).or_default().push(generator);
    }

    pub fn route<FROM, TO>(&mut self)
    where
        FROM: StageSource + Default,
        TO: StageDestination + Default,
        TO::Stage: Stage<Input = <FROM::Stage as Stage>::Output>,
    {
        self.stage(FROM::default(), TO::default())
    }
}

impl Supervisor for Pipeline {
    type GroupBy = ();
}

impl Actor for Pipeline {
    type Context = SupervisorSession<Self>;
}

type RoutePoint<M> = Box<dyn RuntimeGenerator<Input = M>>;
type RouteValue<M> = Vec<RoutePoint<M>>;

pub trait RuntimeGenerator: Send + Sync {
    type Input;

    fn generate(
        &self,
        meta: Metadata,
        pipeline: Address<Pipeline>,
        input: Self::Input,
    ) -> Box<dyn Runtime>;
}

impl Pipeline {
    fn spawn_workers<K, M>(
        &mut self,
        meta: Metadata,
        key: K,
        message: M,
        ctx: &mut SupervisorSession<Pipeline>,
    ) where
        K: TypedMapKey<Value = RouteValue<M>> + Send + Sync + 'static,
        M: Clone + 'static,
    {
        let mut spawned = 0;
        let generators = self.routes.get(&key);
        if let Some(generators) = generators {
            for generator in generators.iter() {
                let pipeline = ctx.address().clone();
                let message = message.clone();
                let runtime = generator.generate(meta, pipeline, message);
                ctx.spawn_trackable(runtime, ());
                spawned += 1;
            }
        }
        if spawned == 0 {
            log::error!(
                "Workers for {} are not presented. Source: {}",
                type_name::<M>(),
                type_name::<K>()
            );
        }
    }
}
