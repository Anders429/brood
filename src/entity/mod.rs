pub(crate) mod allocator;

mod identifier;
mod seal;

pub use identifier::Identifier;

pub(crate) use allocator::Allocator;

use crate::{component::Component, hlist::define_null};
use seal::Seal;

define_null!();

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
