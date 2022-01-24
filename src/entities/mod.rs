mod seal;

use crate::{component::Component, hlist::define_null};
use alloc::vec::Vec;
use seal::Seal;

define_null!();

pub trait Entities: Seal {}

impl Entities for Null {}

impl<C, E> Entities for (Vec<C>, E)
where
    C: Component,
    E: Entities,
{
}

pub struct Batch<E>
where
    E: Entities,
{
    pub(crate) entities: E,
}

impl<E> Batch<E>
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
            $crate::entities::Batch::new_unchecked(
                ($crate::reexports::vec![$component; $n], entities!(@cloned ($($components),*); $n))
            )
        }
    };
    ($(($($components:expr),*)),* $(,)?) => {
        unsafe {
            $crate::entities::Batch::new_unchecked(
                entities!(@transpose [] $(($($components),*)),*)
            )
        }
    };
    ((); $n:expr) => {
        unsafe {
            $crate::entities::Batch::new_unchecked($crate::entities::Null)
        }
    };
    () => {
        unsafe {
            $crate::entities::Batch::new_unchecked($crate::entities::Null)
        }
    };
    (@cloned ($component:expr $(,$components:expr)* $(,)?); $n:expr) => {
        ($crate::reexports::vec![$component; $n], entities!(@cloned ($($components),*); $n))
    };
    (@cloned (); $n:expr) => {
        $crate::entities::Null
    };
    (@transpose [$([$($column:expr),*])*] $(($component:expr $(,$components:expr)*  $(,)?)),*) => {
        entities!(@transpose [$([$($column),*])* [$($component),*]] $(($($components),*)),*)
    };
    (@transpose [$([$($column:expr),*])*] $(()),*) => {
        entities!(@as_vec ($(($($column),*)),*))
    };
    (@as_vec (($($column:expr),*) $(,($($columns:expr),*))* $(,)?)) => {
        ($crate::reexports::vec![$($column),*], entities!(@as_vec ($(($($columns),*)),*)))
    };
    (@as_vec ()) => {
        $crate::entities::Null
    };
}
