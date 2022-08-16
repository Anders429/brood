//! This module defines and implements assertions on a [`Registry`].
//!
//! All assertions on invariants that must be upheld by a `Registry` should be included within the
//! `Assertions` trait defined here. This trait acts as an extension to the `registry::Seal`
//! trait, implementing it on all registries.
//!
//! [`Registry`]: crate::registry::Registry

use crate::{component::Component, registry::Null};
use core::any::TypeId;
use hashbrown::HashSet;

/// Assertions that can be run on a registry to verify that certain invariants are upheld.
pub trait Assertions {
    /// Asserts that no components within the registry are of the same type.
    ///
    /// This is necessary to ensure that logic within a [`World`] is sound, as some logic relies on
    /// the assumption that there is only one component within the registry. Therefore, this
    /// assertion should be made when initializing a new `World` with a registry so that the
    /// registry code internally is sound.
    ///
    /// [`World`]: crate::world::World
    fn assert_no_duplicates(components: &mut HashSet<TypeId, ahash::RandomState>);
}

impl Assertions for Null {
    fn assert_no_duplicates(_components: &mut HashSet<TypeId, ahash::RandomState>) {}
}

impl<C, R> Assertions for (C, R)
where
    C: Component,
    R: Assertions,
{
    fn assert_no_duplicates(components: &mut HashSet<TypeId, ahash::RandomState>) {
        assert!(components.insert(TypeId::of::<C>()));
        R::assert_no_duplicates(components);
    }
}

#[cfg(test)]
mod tests {
    use super::Assertions;
    use crate::registry;
    use hashbrown::HashSet;

    struct A;
    struct B;
    struct C;

    #[test]
    fn no_duplicates() {
        type NoDuplicates = registry!(A, B, C);

        NoDuplicates::assert_no_duplicates(&mut HashSet::with_hasher(ahash::RandomState::new()));
    }

    #[test]
    fn empty_no_duplicates() {
        type Empty = registry!();

        Empty::assert_no_duplicates(&mut HashSet::with_hasher(ahash::RandomState::new()));
    }

    #[test]
    #[should_panic]
    fn has_duplicates() {
        type HasDuplicates = registry!(A, B, A, C);

        HasDuplicates::assert_no_duplicates(&mut HashSet::with_hasher(ahash::RandomState::new()));
    }
}
