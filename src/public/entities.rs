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

// TODO: Bikeshed this name. Yuck.
pub struct EntitiesIter<E>
where
    E: Entities,
{
    pub(crate) entities: E,
}

impl<E> EntitiesIter<E>
where
    E: Entities,
{
    pub fn new(entities: E) -> Self {
        assert!(entities.check_len());
        unsafe { Self::new_unchecked(entities) }
    }

    pub unsafe fn new_unchecked(entities: E) -> Self {
        Self { entities }
    }
}

#[macro_export]
macro_rules! entities {
    (($component:expr $(,$components:expr)* $(,)?); $n:expr) => {
        unsafe {
            $crate::entities::EntitiesIter::new_unchecked(
                ($crate::reexports::vec![$component; $n], entities!(@internal ($($components),*); $n))
            )
        }
    };
    ($($($components:expr),*),*) => {
        // TODO
    };
    ((); $n:expr) => {
        unsafe {
            $crate::entities::EntitiesIter::new_unchecked($crate::entities::NullEntities)
        }
    };
    () => {
        unsafe {
            $crate::entities::EntitiesIter::new_unchecked($crate::entities::NullEntities)
        }
    };
    (@internal ($component:expr $(,$components:expr)* $(,)?); $n:expr) => {
        ($crate::reexports::vec![$component; $n], entities!(@internal ($($components),*); $n))
    };
    (@internal (); $n:expr) => {
        $crate::entities::NullEntities
    };
}
