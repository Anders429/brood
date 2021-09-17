use crate::{
    component::Component,
    entity::{NullEntities, NullEntity},
};
use alloc::vec::Vec;

pub trait EntityLength {
    const LEN: usize;
}

impl EntityLength for NullEntity {
    const LEN: usize = 0;
}

impl<C, E> EntityLength for (C, E)
where
    C: Component,
    E: EntityLength,
{
    const LEN: usize = E::LEN + 1;
}

pub trait EntitiesLength {
    fn component_len(&self) -> usize;
}

impl EntitiesLength for NullEntities {
    fn component_len(&self) -> usize {
        0
    }
}

impl<C, E> EntitiesLength for (Vec<C>, E)
where
    C: Component,
    E: EntitiesLength,
{
    fn component_len(&self) -> usize {
        self.0.len()
    }
}
