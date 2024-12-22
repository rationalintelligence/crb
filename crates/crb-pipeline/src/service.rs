use crate::PhantomData;
use crate::{stage::Stage, RoutePoint, RouteValue, StageSource};
use std::any::type_name;
use std::hash::{Hash, Hasher};
use typedmap::TypedMapKey;

pub struct MessageStage<M> {
    message: M,
}

impl<M> Stage for MessageStage<M>
where
    M: Clone + Send + 'static,
{
    type Input = M;
    type Output = M;

    fn from_input(input: Self::Input) -> Self {
        Self { message: input }
    }

    fn to_output(&mut self) -> Self::Output {
        self.message.clone()
    }
}

pub struct InputStage<M> {
    _type: PhantomData<M>,
}

impl<M> InputStage<M> {
    pub fn stage() -> Self {
        Self { _type: PhantomData }
    }
}

impl<M> StageSource for InputStage<M>
where
    M: Clone + Sync + Send + 'static,
{
    type Stage = MessageStage<M>;
    type Key = InitialKey<M>;

    fn source(&self) -> Self::Key {
        InitialKey::<M>::new()
    }
}

pub struct InitialKey<M> {
    _type: PhantomData<M>,
}

impl<M> InitialKey<M> {
    pub fn new() -> Self {
        Self { _type: PhantomData }
    }
}

impl<M> Clone for InitialKey<M> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<M> Hash for InitialKey<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        type_name::<M>().hash(state);
    }
}

impl<M> PartialEq for InitialKey<M> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<M> Eq for InitialKey<M> {}

impl<M: 'static> TypedMapKey for InitialKey<M> {
    type Value = RouteValue<M>;
}
