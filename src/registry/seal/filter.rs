//! Defines filters that can be applied on this registry.
//!
//! This allows archetype identifiers that are generic over this registry to be safely queried for
//! a component.

use crate::{archetype, registry::Registry};

/// Type marker component's index in registry.
pub enum Index {}

/// Defines a registry as being filterable with a component `C`.
///
/// The indices `I` do not need to provided manually, as they can be inferred by the compiler.
pub trait Filter<C, I> {
    /// Returns whether the identifier has the component `C`.
    fn has<R>(identifier: archetype::IdentifierRef<R>) -> bool
    where
        R: Registry;
}

impl<C, R> Filter<C, Index> for (C, R)
where
    R: Registry,
{
    fn has<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        // SAFETY: `identifier` will have exactly `R_::LEN` bits set. Also, `R_::LEN - R::LEN` will
        // always be at least 1.
        unsafe { identifier.get_unchecked(R_::LEN - R::LEN - 1) }
    }
}

impl<C, C_, I, R> Filter<C_, (I,)> for (C, R)
where
    R: Filter<C_, I>,
{
    fn has<R_>(identifier: archetype::IdentifierRef<R_>) -> bool
    where
        R_: Registry,
    {
        R::has(identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{archetype, registry};
    use alloc::vec;

    struct A;
    struct B;
    struct C;

    type Registry = registry!(A, B, C);

    #[test]
    fn has() {
        assert!(<Registry as Filter<A, _>>::has(unsafe {
            archetype::Identifier::<Registry>::new(vec![1]).as_ref()
        }));
    }

    #[test]
    fn not_has() {
        assert!(!<Registry as Filter<B, _>>::has(unsafe {
            archetype::Identifier::<Registry>::new(vec![1]).as_ref()
        }));
    }

    #[test]
    fn nested_has() {
        assert!(<Registry as Filter<C, _>>::has(unsafe {
            archetype::Identifier::<Registry>::new(vec![4]).as_ref()
        }));
    }
}
