//! A heterogeneous list of registered [`Component`]s.
//!
//! [`Registry`]s are most often defined using the [`registry!`] macro. The items contained within
//! this module should rarely be needed in user code.
//!
//! Recommended practice is to define a `Registry` as a custom type, and use that type when
//! defining a [`World`].
//!
//! # Example
//! ``` rust
//! use brood::{registry, World};
//!
//! // Define components.
//! struct Foo(usize);
//! struct Bar(bool);
//!
//! type Registry = registry!(Foo, Bar);
//!
//! let world = World::<Registry>::new();
//! ```
//!
//! [`Component`]: crate::component::Component
//! [`Registry`]: crate::registry::Registry
//! [`registry!`]: crate::registry!
//! [`World`]: crate::world::World

mod debug;
mod eq;
mod seal;
mod send;
#[cfg(feature = "serde")]
mod serde;
mod sync;

#[cfg(feature = "serde")]
pub(crate) use self::serde::{RegistryDeserialize, RegistrySerialize};
pub(crate) use debug::RegistryDebug;
pub(crate) use eq::{RegistryEq, RegistryPartialEq};
pub(crate) use send::RegistrySend;
pub(crate) use sync::RegistrySync;

use crate::{component::Component, hlist::define_null_uninstantiable};
use seal::Seal;

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
/// use brood::registry;
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// type Registry = registry!(Foo, Bar);
/// ```
///
/// [`Component`]: crate::component::Component
/// [`World`]: crate::World
pub trait Registry: Seal {}

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
/// use brood::{registry, World};
///
/// // Define components `Foo` and `Bar`.
/// struct Foo(u16);
/// struct Bar(f32);
///
/// // Define a registry containing those components.
/// type Registry = registry!(Foo, Bar);
///
/// // Define a world using the registry.
/// let world = World::<Registry>::new();
/// ```
///
/// [`World`]: crate::World
#[macro_export]
macro_rules! registry {
    ($component:ty $(,$components:ty)* $(,)?) => {
        ($component, $crate::registry!($($components,)*))
    };
    () => {
        $crate::registry::Null
    };
}
