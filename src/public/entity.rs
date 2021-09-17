use crate::{component::Component, internal::entity::{EntitiesSeal, EntitySeal}};
use core::any::Any;
use alloc::vec::Vec;

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

pub struct NullEntities;

pub trait Entities: EntitiesSeal {}

impl Entities for NullEntities {}

impl<C, E> Entities for (Vec<C>, E) where C: Component, E: Entities {}

#[macro_export]
macro_rules! entities {
    (($component:expr $(,$components:expr)* $(,)?); $n:expr) => {
        ($crate::reexports::vec![$component; $n], entities!(($($components),*); $n))
    };
    ($($($components:expr),*),*) => {};
    ((); $n:expr) => {
        $crate::entity::NullEntities
    };
    () => {
        $crate::entity::NullEntities
    };
}
