mod storage;

use crate::{component::Component, entity::Null};
use storage::EntityStorage;

pub trait EntitySeal: EntityStorage {}

impl EntitySeal for Null {}

impl<C, E> EntitySeal for (C, E)
where
    C: Component,
    E: EntitySeal,
{
}
