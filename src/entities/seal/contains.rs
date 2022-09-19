//! Provides the `Contains` trait to indicate what entity type is contained in a heterogeneous list
//! implementing `Entities`.

use crate::{component::Component, entities::Null, entity, entity::Entity};
use alloc::vec::Vec;

/// Defines the entity type contained by `Entities`.
pub trait Contains {
    /// The type of entity contained.
    type Entity: Entity;
}

impl Contains for Null {
    type Entity = entity::Null;
}

impl<C, E> Contains for (Vec<C>, E)
where
    C: Component,
    E: Contains,
{
    type Entity = (C, E::Entity);
}
