//! Provides a `ContainsComponent` trait to indicate that a registry contains a component.

mod sealed;

use sealed::Sealed;

/// Indicates that the component `C` is contained in the registry.
pub trait ContainsComponent<C, I>: Sealed<C, I> {}

impl<T, C, I> ContainsComponent<C, I> for T where T: Sealed<C, I> {}
