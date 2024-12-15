use crate::Actor;

struct ActorRuntime<T: Actor> {
    actor: T,
    context: T::Context,
}
