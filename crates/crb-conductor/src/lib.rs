pub mod actor;

pub use actor::ConductedActor;

use crb_actor::Actor;

pub trait Conductor: Actor {}
