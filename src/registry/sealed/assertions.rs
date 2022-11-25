//! This module defines and implements assertions on a [`Registry`].
//!
//! All assertions on invariants that must be upheld by a `Registry` should be included within the
//! `Assertions` trait defined here. This trait acts as an extension to the `registry::Seal`
//! trait, implementing it on all registries.
//!
//! [`Registry`]: crate::registry::Registry

use crate::{
    component::Component,
    registry::Null,
};
use core::any::TypeId;
use fnv::FnvBuildHasher;
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
    fn assert_no_duplicates(components: &mut HashSet<TypeId, FnvBuildHasher>);
}

impl Assertions for Null {
    fn assert_no_duplicates(_components: &mut HashSet<TypeId, FnvBuildHasher>) {}
}

impl<C, R> Assertions for (C, R)
where
    C: Component,
    R: Assertions,
{
    fn assert_no_duplicates(components: &mut HashSet<TypeId, FnvBuildHasher>) {
        assert!(components.insert(TypeId::of::<C>()));
        R::assert_no_duplicates(components);
    }
}

#[cfg(test)]
mod tests {
    use super::Assertions;
    use crate::Registry;
    use fnv::FnvBuildHasher;
    use hashbrown::HashSet;

    struct A;
    struct B;
    struct C;

    #[test]
    fn no_duplicates() {
        type NoDuplicates = Registry!(A, B, C);

        NoDuplicates::assert_no_duplicates(&mut HashSet::with_hasher(FnvBuildHasher::default()));
    }

    #[test]
    fn empty_no_duplicates() {
        type Empty = Registry!();

        Empty::assert_no_duplicates(&mut HashSet::with_hasher(FnvBuildHasher::default()));
    }

    #[test]
    #[should_panic]
    fn has_duplicates() {
        type HasDuplicates = Registry!(A, B, A, C);

        HasDuplicates::assert_no_duplicates(&mut HashSet::with_hasher(FnvBuildHasher::default()));
    }
}
