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

/// Creates a batch of entities made from the same components.
///
/// This macro allows multiple entities to be defined for efficient storage within a [`World`]. The
/// syntax is similar to an array expression or the [`vec!`] macro. There are two forms of this
/// macro:
///
/// # Create entities given a list of tuples of components
///
/// ``` rust
/// use brood::entities;
///
/// // Define components `Foo` and `Bar`.
/// struct Foo(u16);
/// struct Bar(f32);
///
/// let my_entities = entities![(Foo(42), Bar(1.5)), (Foo(4), Bar(1.0))];
/// ```
///
/// Note that all tuples must be made from the same components in the same order.
///
/// # Create entities from a tuple of components and a size
///
/// ``` rust
/// use brood::entities;
///
/// // Define components `Foo` and `Bar`.
/// #[derive(Clone)]
/// struct Foo(u16);
///
/// #[derive(Clone)]
/// struct Bar(f32);
///
/// let my_entities = entities![(Foo(42), Bar(1.5)); 3];
/// ```
///
/// This syntax only supports entities made from components that implement [`Clone`], and the size
/// does not have to be a constant value.
///
/// This will call `clone()` to duplicate each component, so one should be careful with types that
/// have a nonstandard `Clone` implementation. For example, using `Rc` as a component value in this
/// context will create multiple entities with references to the same boxed value, not multiple
/// references to independently boxed values.
///
/// Using `0` as a size is allowed, and produces a container of no entities. This will still
/// evaluate all expressions passed as components, however, and immediately drop the resulting
/// values, so be mindful of side effects.
///
/// [`Clone`]: core::clone::Clone
/// [`World`]: crate::World
/// [`vec!`]: alloc::vec!
#[macro_export]
macro_rules! entities {
    (($component:expr $(,$components:expr)* $(,)?); $n:expr) => {
        unsafe {
            $crate::entities::Batch::new_unchecked(
                ($crate::reexports::vec![$component; $n], $crate::entities!(@cloned ($($components),*); $n))
            )
        }
    };
    ($(($($components:expr),*)),* $(,)?) => {
        unsafe {
            $crate::entities::Batch::new_unchecked(
                $crate::entities!(@transpose [] $(($($components),*)),*)
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
        ($crate::reexports::vec![$component; $n], $crate::entities!(@cloned ($($components),*); $n))
    };
    (@cloned (); $n:expr) => {
        $crate::entities::Null
    };
    (@transpose [$([$($column:expr),*])*] $(($component:expr $(,$components:expr)*  $(,)?)),*) => {
        $crate::entities!(@transpose [$([$($column),*])* [$($component),*]] $(($($components),*)),*)
    };
    (@transpose [$([$($column:expr),*])*] $(()),*) => {
        $crate::entities!(@as_vec ($(($($column),*)),*))
    };
    (@as_vec (($($column:expr),*) $(,($($columns:expr),*))* $(,)?)) => {
        ($crate::reexports::vec![$($column),*], $crate::entities!(@as_vec ($(($($columns),*)),*)))
    };
    (@as_vec ()) => {
        $crate::entities::Null
    };
}
