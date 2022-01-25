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

#[macro_export]
macro_rules! entity {
    ($component:expr $(,$components:expr)* $(,)?) => {
        ($component, $crate::entity!($($components,)*))
    };
    () => {
        $crate::entity::Null
    };
}
