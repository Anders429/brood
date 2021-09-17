mod byte_length;
mod length;
mod storage;

use crate::{component::Component, entity::{NullEntities, NullEntity}};
use alloc::vec::Vec;
use byte_length::EntityByteLength;
use length::{EntitiesLength, EntityLength};
use storage::{EntitiesStorage, EntityStorage};

pub trait EntitySeal: EntityByteLength + EntityLength + EntityStorage {}

impl EntitySeal for NullEntity {}

impl<C, E> EntitySeal for (C, E)
where
    C: Component,
    E: EntitySeal,
{
}

pub trait EntitiesSeal: EntitiesLength + EntitiesStorage {}

impl EntitiesSeal for NullEntities {}

impl<C, E> EntitiesSeal for (Vec<C>, E) where C: Component, E: EntitiesSeal, {}
