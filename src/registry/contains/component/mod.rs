//! Provides a `ContainsComponent` trait to indicate that a registry contains a component.

mod sealed;

use sealed::Sealed;

/// Indicates that a component is contained in the registry.
pub trait ContainsComponent<Component, Index>: Sealed<Component, Index> {}

impl<Registry, Component, Index> ContainsComponent<Component, Index> for Registry where
    Registry: Sealed<Component, Index>
{
}
