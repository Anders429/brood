//! A heterogeneous list of registered [`Component`]s.
//!
//! [`Registry`]s are most often defined using the [`Registry!`] macro. The items contained within
//! this module should rarely be needed in user code.
//!
//! Recommended practice is to define a `Registry` as a custom type, and use that type when
//! defining a [`World`].
//!
//! # Example
//! ``` rust
//! use brood::{
//!     Registry,
//!     World,
//! };
//!
//! // Define components.
//! struct Foo(usize);
//! struct Bar(bool);
//!
//! type Registry = Registry!(Foo, Bar);
//!
//! let world = World::<Registry>::new();
//! ```
//!
//! [`Component`]: crate::component::Component
//! [`Registry`]: crate::registry::Registry
//! [`Registry!`]: crate::Registry!
//! [`World`]: crate::world::World

pub(crate) mod contains;

mod debug;
mod eq;
mod sealed;
#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "serde")]
pub use self::serde::{
    Deserialize,
    Serialize,
};
#[cfg(feature = "rayon")]
pub use contains::ContainsParQuery;
pub use contains::{
    ContainsComponent,
    ContainsEntities,
    ContainsEntity,
    ContainsQuery,
};
pub use debug::Debug;
pub use eq::{
    Eq,
    PartialEq,
};

#[cfg(feature = "rayon")]
pub(crate) use contains::ContainsParViews;
pub(crate) use contains::ContainsViews;
#[cfg(feature = "rayon")]
pub(crate) use sealed::CanonicalParViews;
pub(crate) use sealed::{
    Canonical,
    CanonicalViews,
    Length,
};

use crate::{
    component::Component,
    hlist::define_null_uninstantiable,
};
use sealed::Sealed;

define_null_uninstantiable!();

/// A heterogeneous list of [`Component`]s.
///
/// Registries are used when defining [`World`]s. In order for components to be stored within a
/// `World`, they must be included in the `World`'s registry. However, care should be made to only
/// include `Component`s that will be used, as unused `Component`s will cause unnecessary heap
/// allocations.
///
/// While duplicate `Component`s can be included within a registry, it is not advised. There are no
/// benefits to including multiple `Component`s, and the unused components cause higher memory
/// allocation within a `World`.
///
/// # Example
/// ``` rust
/// use brood::Registry;
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// type Registry = Registry!(Foo, Bar);
/// ```
///
/// [`Component`]: crate::component::Component
/// [`World`]: crate::World
pub trait Registry: Sealed + 'static {}

impl Registry for Null {}

impl<C, R> Registry for (C, R)
where
    C: Component,
    R: Registry,
{
}

/// Creates a registry from the provided components.
///
/// This macro allows a registry to be defined without needing to manually create a heterogeneous
/// list of components. Given an arbitrary number of components, this macro will arrange them into
/// nested tuples forming a heterogeneous list.
///
/// A registry is not normally instantiated. Its main purpose is to be used as a generic in the
/// definition of a [`World`].
///
/// # Example
/// ``` rust
/// use brood::{
///     Registry,
///     World,
/// };
///
/// // Define components `Foo` and `Bar`.
/// struct Foo(u16);
/// struct Bar(f32);
///
/// // Define a registry containing those components.
/// type Registry = Registry!(Foo, Bar);
///
/// // Define a world using the registry.
/// let world = World::<Registry>::new();
/// ```
///
/// [`World`]: crate::World
#[macro_export]
macro_rules! Registry {
    ($component:ty $(,$components:ty)* $(,)?) => {
        ($component, $crate::Registry!($($components,)*))
    };
    () => {
        $crate::registry::Null
    };
}
