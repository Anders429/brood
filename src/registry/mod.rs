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

use crate::{component::Component, hlist::define_null};
use seal::Seal;

define_null!();

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
