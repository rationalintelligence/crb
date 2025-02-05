use crate::agent::Agent;

/// Stateful extenions for the `Context`.
pub trait ExtensionFor<A: Agent>: Send + 'static {
    type View<'a>;

    fn extend(&mut self, ctx: &mut A::Context) -> Self::View<'_>;
}
