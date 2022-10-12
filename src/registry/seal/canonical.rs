//! Defines entities which are canonical within a registry.
//!
//! A canonical entity is an entity whose components are ordered the same as a registry. Therefore,
//! the canonicality relationship is defined as a trait implemented on a registry, generic over an
//! entity type. Any entity `E` that is canonical with respect to a registry will have that
//! relationship encoded by an `impl Canonical<E, P>`, for some `P` inferred by the compiler.

use crate::{
    archetype,
    component::Component,
    entity,
    registry,
    registry::{
        seal::Length,
        Registry,
    },
};
use alloc::{
    vec,
    vec::Vec,
};

/// Type marker for a component contained in an entity.
pub enum Contained {}

/// Type marker for a component not contained in an entity.
pub enum NotContained {}

/// Defines either a null containment or a null index.
pub enum Null {}

/// Functions implemented on entities `E` that are canonical in terms of the given registry.
///
/// Using canonical entities, we are able to easily create archetype identifiers for archetypes
/// storing entities of type `E`.
///
/// This trait will not be implemented for an `E` that is not a canonical entity in terms of the
/// given registry, which guarantees that archetype identifiers created by it will always be valid.
pub trait Canonical<E, P>: Registry + Sized {
    /// Safely create an archetype identifier for the entity `E`.
    ///
    /// This is guaranteed to be valid, since the entity `E` is always canonical in terms of this
    /// registry.
    #[must_use]
    fn create_archetype_identifier() -> archetype::Identifier<Self>;

    /// Populate the raw buffer of an archetype identifier.
    ///
    /// All components contained in the canonical entity `E` will have their respective bits set in
    /// `identifier`.
    ///
    /// The safe interface for this functionality is the `create_archetype_identifier()` method.
    ///
    /// # Safety
    /// `identifier` must be a properly-initialized buffer containing at least `(length + 7) / 8`
    /// bytes.
    unsafe fn populate_archetype_identifier(identifier: &mut [u8], length: usize);
}

impl Canonical<entity::Null, Null> for registry::Null {
    fn create_archetype_identifier() -> archetype::Identifier<Self> {
        // SAFETY: The length of an empty registry is 0, so the identifier is built from a buffer
        // of 0 bytes.
        unsafe { archetype::Identifier::new(Vec::new()) }
    }

    unsafe fn populate_archetype_identifier(_identifier: &mut [u8], _length: usize) {}
}

impl<C, E, P, R> Canonical<(C, E), (Contained, P)> for (C, R)
where
    C: Component,
    R: Canonical<E, P>,
{
    fn create_archetype_identifier() -> archetype::Identifier<Self> {
        let mut raw_identifier = vec![0; (Self::LEN + 7) / 8];

        // SAFETY: `raw_identifier` is a properly-initialized buffer containing `(R::LEN + 7) / 8`
        // bytes.
        unsafe {
            <Self as Canonical<(C, E), (Contained, P)>>::populate_archetype_identifier(
                &mut raw_identifier,
                Self::LEN - 1,
            );
            archetype::Identifier::new(raw_identifier)
        }
    }

    unsafe fn populate_archetype_identifier(identifier: &mut [u8], length: usize) {
        // SAFETY: Since `identifier` contains at least `(length + 7) / 8` bytes, this indexing
        // into `identifier` is guaranteed to be valid.
        *unsafe { identifier.get_unchecked_mut((length - R::LEN) / 8) } |=
            1 << ((length - R::LEN) % 8);
        // SAFETY: Since the safety contract of the current function is the same as the contract of
        // this function call, this is safe to call.
        unsafe {
            R::populate_archetype_identifier(identifier, length);
        }
    }
}

impl<C, E, P, R> Canonical<E, (NotContained, P)> for (C, R)
where
    C: Component,
    R: Canonical<E, P>,
{
    fn create_archetype_identifier() -> archetype::Identifier<Self> {
        let mut raw_identifier = vec![0; (Self::LEN + 7) / 8];

        // SAFETY: `raw_identifier` is a properly-initialized buffer containing `(R::LEN + 7) / 8`
        // bytes.
        unsafe {
            <Self as Canonical<E, (NotContained, P)>>::populate_archetype_identifier(
                &mut raw_identifier,
                Self::LEN - 1,
            );
            archetype::Identifier::new(raw_identifier)
        }
    }

    unsafe fn populate_archetype_identifier(identifier: &mut [u8], length: usize) {
        // SAFETY: Since the safety contract of the current function is the same as the contract of
        // this function call, this is safe to call.
        unsafe {
            R::populate_archetype_identifier(identifier, length);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        archetype,
        entity,
        registry,
    };
    use alloc::vec::Vec;

    struct A;
    struct B;
    struct C;
    struct D;
    struct E;
    struct F;
    struct G;
    struct H;
    struct I;

    // Using enough components to create archetype identifiers with more than one byte.
    type Registry = registry!(A, B, C, D, E, F, G, H, I);

    #[test]
    fn create_archetype_identifier_empty_registry() {
        type Registry = registry!();

        assert_eq!(Registry::create_archetype_identifier(), unsafe {
            archetype::Identifier::<Registry>::new(Vec::new())
        });
    }

    #[test]
    fn create_archetype_identifier_all_components() {
        assert_eq!(
            <Registry as Canonical<(A, (B, (C, (D, (E, (F, (G, (H, (I, entity::Null))))))))), _>>::create_archetype_identifier(),
            unsafe {archetype::Identifier::<Registry>::new(vec![255, 1])}
        );
    }

    #[test]
    fn create_archetype_identifier_first_component_present() {
        assert_eq!(
            <Registry as Canonical<(A, (C, (D, entity::Null))), _>>::create_archetype_identifier(),
            unsafe { archetype::Identifier::<Registry>::new(vec![13, 0]) }
        );
    }

    #[test]
    fn create_archetype_identifier_first_component_not_present() {
        assert_eq!(
            <Registry as Canonical<(B, (E, (I, entity::Null))), _>>::create_archetype_identifier(),
            unsafe { archetype::Identifier::<Registry>::new(vec![18, 1]) }
        );
    }
}
