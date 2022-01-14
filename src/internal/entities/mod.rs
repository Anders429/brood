mod length;
mod storage;

use crate::{component::Component, entities::Null};
use alloc::vec::Vec;
use length::EntitiesLength;
use storage::EntitiesStorage;

pub trait EntitiesSeal: EntitiesLength + EntitiesStorage {}

impl EntitiesSeal for Null {}

impl<C, E> EntitiesSeal for (Vec<C>, E)
where
    C: Component,
    E: EntitiesSeal,
{
}
