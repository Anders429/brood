//! A heterogeneous list of batches of [`Component`]s stored within a [`World`].
//!
//! A [`Batch`] of [`Entities`] is a container of entities all made of the same combination of
//! `Component`s. Inserting a `Batch` of `Entities` is much more efficient than storing them one at
//! a time.
//!
//! `Batch`es of entities are most often defined using the [`entities!`] macro. The items
//! contained within this module should rarely be needed in user code.
//!
//! `Entity`s are stored within [`World`]s to allow efficient querying and iteration with other
//! entities of similar components. Since entities are defined as heterogeneous lists, they can be
//! made of an arbitrary number of components. `World`s can store entities made up of any
//! combination of components, so long as those components are stored in the `World`'s
//! [`Registry`].
//!
//! # Example
//! ``` rust
//! use brood::{entities, registry, World};
//!
//! // Define components.
//! struct Foo(usize);
//! struct Bar(bool);
//!
//! type Registry = registry!(Foo, Bar);
//!
//! let mut world = World::<Registry>::new();
//!
//! world.extend(entities!((Foo(42), Bar(false)), (Foo(100), Bar(true))));
//! ```
//!
//! [`Batch`]: crate::entities::Batch
//! [`Component`]: crate::component::Component
//! [`Entities`]: crate::entities::Entities
//! [`entities!`]: crate::entities!
//! [`Registry`]: crate::registry::Registry
//! [`World`]: crate::world::World

mod seal;

use crate::{component::Component, hlist::define_null};
use alloc::vec::Vec;
use seal::Seal;

define_null!();

/// A heterogeneous list of columns of [`Component`]s.
///
/// In order for `Entities` to be able to be used, it must be contained within a [`Batch`]. This
/// guarnatees that the length of all columns within the heterogeneous list are equal.
///
/// Entities are stored within [`World`]s. In order for an entity to be able to be stored within a
/// `World`, that `World`'s [`Registry`] must include the `Component`s that make up an entity.
///
/// Note that entities must consist of unique component types. Duplicate components are not
/// supported. When multiple components of the same type are included in an entity, a `World` will
/// only store one of those components.
///
/// # Example
/// ``` rust
/// use brood::entities;
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// // Creates `Entities` wrapped in a `Batch` struct.
/// let entities = entities!((Foo(42), Bar(true)), (Foo(100), Bar(false)));
/// ```
///
/// [`Batch`]: crate::entities::Batch
/// [`Component`]: crate::component::Component
/// [`Registry`]: crate::registry::Registry
/// [`World`]: crate::world::World
pub trait Entities: Seal {}

impl Entities for Null {}

impl<C, E> Entities for (Vec<C>, E)
where
    C: Component,
    E: Entities,
{
}

/// A batch of entity columns of unified length.
///
/// This is a wrapper for an [`Entities`] heterogeneous list of columns of components, with the
/// guarantee that all columns are of the same length. In other words, this is a collection of
/// entities separated column-wise into their components.
///
/// A `Batch` is most often created using the [`entities!`] macro.
///
/// # Example
/// ``` rust
/// use brood::entities;
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// // This defines a `Batch` of entities, with guaranteed equal column length.
/// let entities = entities!((Foo(42), Bar(false)), (Foo(100), Bar(true)));
/// ```
///
/// [`Entities`]: crate::entities::Entities
/// [`entities!`]: crate::entities!
pub struct Batch<E>
where
    E: Entities,
{
    pub(crate) entities: E,
    len: usize,
}

impl<E> Batch<E>
where
    E: Entities,
{
    /// Creates a new `Batch`, wrapping the given [`Entities`] heterogeneous list.
    ///
    /// This method performs a run-time check to ensure the columns are all the same length.
    ///
    /// Note that this method can only be called using a raw heterogeneous list of entity columns.
    /// There is currently no macro that can make the call simpler. It is recommended to use the
    /// [`entities!`] macro instead of this method.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{entities, entities::Batch};
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// let batch = Batch::new((vec![42; 10], (vec![true; 10], entities::Null)));
    /// ```
    ///
    /// [`Entities`]: crate::entities::Entities
    /// [`entities!]: crate::entities!
    ///
    /// # Panics
    /// Panics if the columns are not all the same length.
    pub fn new(entities: E) -> Self {
        assert!(entities.check_len());
        unsafe { Self::new_unchecked(entities) }
    }

    /// Creates a new `Batch`, wrapping the given [`Entities`] heterogeneous list, skipping any
    /// run-time checks for column length.
    ///
    /// Note that this method can only be called using a raw heterogeneous list of entity columns.
    /// There is currently no macro that can make the call simpler. It is recommended to use the
    /// [`entities!`] macro instead of this method.
    ///
    /// # Safety
    /// The caller must guarantee that the lengths of all columns within `entities` are equal.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{entities, entities::Batch};
    ///
    /// // Define components.
    /// struct Foo(usize);
    /// struct Bar(bool);
    ///
    /// let batch = unsafe { Batch::new_unchecked((vec![42; 10], (vec![true; 10], entities::Null))) };
    /// ```
    /// [`Entities`]: crate::entities::Entities
    /// [`entities!]: crate::entities!
    pub unsafe fn new_unchecked(entities: E) -> Self {
        Self {
            len: entities.component_len(),
            entities,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
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
    ($(($($components:expr),*)),+ $(,)?) => {
        unsafe {
            $crate::entities::Batch::new_unchecked(
                $crate::entities!(@transpose [] $(($($components),*)),+)
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
