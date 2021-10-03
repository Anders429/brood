mod identifier;

pub use identifier::EntityIdentifier;

use crate::{component::Component, internal::entity::EntitySeal};
use core::any::Any;

pub struct NullEntity;

pub trait Entity: EntitySeal + Any {}

impl Entity for NullEntity {}

impl<C, E> Entity for (C, E)
where
    C: Component,
    E: Entity,
{
}

#[macro_export]
macro_rules! entity {
    ($component:expr $(,$components:expr)* $(,)?) => {
        ($component, entity!($($components,)*))
    };
    () => {
        $crate::entity::NullEntity
    };
}
