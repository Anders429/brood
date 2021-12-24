mod storage;

use crate::{component::Component, entity::NullEntity};
use storage::EntityStorage;

pub trait EntitySeal: EntityStorage {}

impl EntitySeal for NullEntity {}

impl<C, E> EntitySeal for (C, E)
where
    C: Component,
    E: EntitySeal,
{
}
