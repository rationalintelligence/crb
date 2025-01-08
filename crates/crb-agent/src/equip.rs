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
