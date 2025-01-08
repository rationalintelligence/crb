use crate::{Address, Agent};

pub trait Equipment {
    type Agent: Agent;

    fn from(address: Address<Self::Agent>) -> Self;
}

pub trait Equip<E> {
    fn equip(self) -> E;
}

impl<E, A> Equip<E> for Address<A>
where
    A: Agent,
    E: Equipment<Agent = A>,
{
    fn equip(self) -> E {
        E::from(self)
    }
}
