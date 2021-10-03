use crate::{component::Component, entities::NullEntities};
use alloc::vec::Vec;

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
