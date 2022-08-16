//! A buffer for performing assertions on [`Views`].
//!
//! This module provides a reusable bufer for performing assertions on views. This buffer should be
//! used any time `Views` are used to query data from a [`World`], as not performing the assertion
//! is not sound.
//!
//! [`Views`]: crate::query::view::Views
//! [`World`]: crate::world::World

use crate::component::Component;
use core::any::{type_name, TypeId};
use hashbrown::HashSet;

/// A buffer for performing assertions on [`Views`].
///
/// This struct is intended to be used as a reusable buffer for ensuring that `Views` don't attempt
/// to borrow components in a way that will cause undefined behavior. Specifically, it ensures that
/// each component is either only borrowed once mutably, or is borrowed multiple times immutably.
/// If any claim run through the buffer breaks these rules, a panic will occur.
///
/// It is recommended to create a single buffer with the capacity of a registry and reuse it for
/// every check to avoid unnecessary allocations.
///
/// In the future, it would be ideal to move this check to compile time. However, running a check
/// like this using const functions on a heterogeneous list is just not possible yet without a
/// large amount of unstable nightly features.
///
/// [`Views`]: crate::query::view::Views
pub struct AssertionBuffer {
    /// Components that are viewed mutably.
    mutable_claims: HashSet<TypeId, ahash::RandomState>,
    /// Components that are viewed immutably.
    immutable_claims: HashSet<TypeId, ahash::RandomState>,
}

impl AssertionBuffer {
    /// Create a new empty buffer.
    ///
    /// It is recommended to use [`with_capacity`] if possible, as it will save allocations.
    ///
    /// [`with_capacity`]: AssertionBuffer::with_capacity()
    #[cfg(feature = "rayon")]
    pub(crate) fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Create a new buffer with the given capacity.
    ///
    /// A buffer created this way will have the capacity to ensure this number of components are
    /// viewed correctly without creating any extra allocations. It is recommended to call this
    /// method with the size of the [`World`]'s [`Registry`] to enable storing all components of
    /// the `Registry`.
    ///
    /// [`Registry`]: crate::registry::Registry
    /// [`World`]: crate::world::World
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            mutable_claims: HashSet::with_capacity_and_hasher(capacity, ahash::RandomState::new()),
            immutable_claims: HashSet::with_capacity_and_hasher(
                capacity,
                ahash::RandomState::new(),
            ),
        }
    }

    /// Claim a component as being viewed mutably.
    ///
    /// # Panics
    /// This method will panic if the component is already claimed, either mutably or immutably.
    pub(crate) fn claim_mutable<C>(&mut self)
    where
        C: Component,
    {
        assert!(
            !self.mutable_claims.contains(&TypeId::of::<C>()),
            "the component {} cannot be viewed as mutable when it is already viewed as mutable",
            type_name::<C>()
        );
        assert!(
            !self.immutable_claims.contains(&TypeId::of::<C>()),
            "the component {} cannot be viewed as mutable when it is already viewed as immutable",
            type_name::<C>()
        );

        self.mutable_claims.insert(TypeId::of::<C>());
    }

    /// Claim a component as being viewed immutably.
    ///
    /// # Panics
    /// This method will panic if the component is already claimed mutably. Note that components
    /// may be claimed immutably more than once.
    pub(crate) fn claim_immutable<C>(&mut self)
    where
        C: Component,
    {
        assert!(
            !self.mutable_claims.contains(&TypeId::of::<C>()),
            "the component {} cannot be viewed as immutable when it is already viewed as mutable",
            type_name::<C>()
        );

        self.immutable_claims.insert(TypeId::of::<C>());
    }

    /// Clear the buffer, allowing it to be reused.
    ///
    /// It is recommended to clear the buffer and reuse it instead of creating a new one, as this
    /// will prevent unnecessary allocations.
    pub(crate) fn clear(&mut self) {
        self.mutable_claims.clear();
        self.immutable_claims.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::AssertionBuffer;

    // Test components.
    struct A;
    struct B;

    #[cfg(feature = "rayon")]
    #[test]
    fn new() {
        let buffer = AssertionBuffer::new();

        assert!(buffer.mutable_claims.is_empty());
        assert!(buffer.immutable_claims.is_empty());
    }

    #[test]
    fn with_capacity() {
        let buffer = AssertionBuffer::with_capacity(42);

        assert!(buffer.mutable_claims.capacity() >= 42);
        assert!(buffer.immutable_claims.capacity() >= 42);
    }

    #[test]
    fn claim_mutable() {
        let mut buffer = AssertionBuffer::with_capacity(1);

        buffer.claim_mutable::<A>();
    }

    #[test]
    fn claim_immutable() {
        let mut buffer = AssertionBuffer::with_capacity(1);

        buffer.claim_immutable::<A>();
    }

    #[test]
    fn claim_immutable_multiple_times() {
        let mut buffer = AssertionBuffer::with_capacity(1);

        buffer.claim_immutable::<A>();
        buffer.claim_immutable::<A>();
    }

    #[test]
    #[should_panic]
    fn claim_mutable_multiple_times() {
        let mut buffer = AssertionBuffer::with_capacity(1);

        buffer.claim_mutable::<A>();
        buffer.claim_mutable::<A>();
    }

    #[test]
    #[should_panic]
    fn claim_mutable_than_immutable() {
        let mut buffer = AssertionBuffer::with_capacity(1);

        buffer.claim_mutable::<A>();
        buffer.claim_immutable::<A>();
    }

    #[test]
    #[should_panic]
    fn claim_immutable_than_mutable() {
        let mut buffer = AssertionBuffer::with_capacity(1);

        buffer.claim_immutable::<A>();
        buffer.claim_mutable::<A>();
    }

    #[test]
    fn claim_multiple_components_mutable() {
        let mut buffer = AssertionBuffer::with_capacity(2);

        buffer.claim_mutable::<A>();
        buffer.claim_mutable::<B>();
    }

    #[test]
    fn claim_multiple_components_immutable() {
        let mut buffer = AssertionBuffer::with_capacity(2);

        buffer.claim_immutable::<A>();
        buffer.claim_immutable::<B>();
    }

    #[test]
    fn claim_multiple_components_mutable_and_immutable() {
        let mut buffer = AssertionBuffer::with_capacity(2);

        buffer.claim_mutable::<A>();
        buffer.claim_immutable::<B>();
    }

    #[test]
    fn clear_empty() {
        let mut buffer = AssertionBuffer::with_capacity(0);

        buffer.clear();
    }

    #[test]
    fn clear_mutable_claims() {
        let mut buffer = AssertionBuffer::with_capacity(1);
        buffer.claim_mutable::<A>();

        buffer.clear();

        buffer.claim_mutable::<A>();
    }

    #[test]
    fn clear_immutable_claims() {
        let mut buffer = AssertionBuffer::with_capacity(1);
        buffer.claim_immutable::<A>();

        buffer.clear();

        buffer.claim_mutable::<A>();
    }

    #[test]
    fn clear_mutable_and_immutable_claims() {
        let mut buffer = AssertionBuffer::with_capacity(2);
        buffer.claim_immutable::<A>();
        buffer.claim_mutable::<B>();

        buffer.clear();

        buffer.claim_mutable::<A>();
        buffer.claim_immutable::<B>();
    }
}
