mod length;
mod storage;

use crate::{component::Component, entities::NullEntities};
use alloc::vec::Vec;
use length::EntitiesLength;
use storage::EntitiesStorage;

pub trait EntitiesSeal: EntitiesLength + EntitiesStorage {}

impl EntitiesSeal for NullEntities {}

impl<C, E> EntitiesSeal for (Vec<C>, E)
where
    C: Component,
    E: EntitiesSeal,
{
}
