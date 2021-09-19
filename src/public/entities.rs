use crate::{component::Component, internal::entities::EntitiesSeal};
use alloc::vec::Vec;

pub struct NullEntities;

pub trait Entities: EntitiesSeal {}

impl Entities for NullEntities {}

impl<C, E> Entities for (Vec<C>, E)
where
    C: Component,
    E: Entities,
{
}

#[macro_export]
macro_rules! entities {
    (($component:expr $(,$components:expr)* $(,)?); $n:expr) => {
        ($crate::reexports::vec![$component; $n], entities!(($($components),*); $n))
    };
    ($($($components:expr),*),*) => {};
    ((); $n:expr) => {
        $crate::entities::NullEntities
    };
    () => {
        $crate::entities::NullEntities
    };
}