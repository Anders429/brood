pub(crate) mod allocator;

mod identifier;
mod seal;

pub use identifier::Identifier;

pub(crate) use allocator::Allocator;

use crate::{component::Component, hlist::define_null};
use seal::Seal;

define_null!();

/// A heterogeneous list of [`Component`]s.
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
/// use brood::entity;
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// let entity = entity!(Foo(42), Bar(true));
/// ```
///
/// [`Component`]: crate::component::Component
/// [`Registry`]: crate::registry::Registry
/// [`World`]: crate::world::World
pub trait Entity: Seal {}

impl Entity for Null {}

impl<C, E> Entity for (C, E)
where
    C: Component,
    E: Entity,
{
}

/// Creates an entity from the provided components.
///
/// This macro allows an enity to be defined without needing to manually create a heterogeneous
/// list of components. Given an arbitrary number of components, this macro will arrange them into
/// nested tuples forming a heterogeneous list.
///
/// # Example
/// ``` rust
/// use brood::entity;
///
/// // Define components `Foo` and `Bar`.
/// struct Foo(u16);
/// struct Bar(f32);
///
/// // Define an entity containing an instance of `Foo` and `Bar`.
/// let my_entity = entity!(Foo(42), Bar(1.5));
/// ```
#[macro_export]
macro_rules! entity {
    ($component:expr $(,$components:expr)* $(,)?) => {
        ($component, $crate::entity!($($components,)*))
    };
    () => {
        $crate::entity::Null
    };
}
