use crate::{Address, Agent};

pub trait Equip<A: Agent> {
    fn equip<E>(self) -> E
    where
        E: From<Address<A>>;
}

impl<A> Equip<A> for Address<A>
where
    A: Agent,
{
    fn equip<E>(self) -> E
    where
        E: From<Address<A>>,
    {
        E::from(self)
    }
}

/// This implementation is useful for using with supervisors that can return
/// addresses of spawned agents with assigned relations in a tuple.
impl<A, X> Equip<A> for (Address<A>, X)
where
    A: Agent,
{
    fn equip<E>(self) -> E
    where
        E: From<Address<A>>,
    {
        E::from(self.0)
    }
}
