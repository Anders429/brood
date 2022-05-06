mod impl_debug;
mod impl_eq;
#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;
mod iter;
#[cfg(feature = "parallel")]
mod par_iter;

#[cfg(feature = "serde")]
pub(crate) use impl_serde::DeserializeArchetypes;
pub(crate) use iter::IterMut;
#[cfg(feature = "parallel")]
pub(crate) use par_iter::ParIterMut;

use crate::{archetype, archetype::Archetype, entity, registry::Registry};
use core::hash::{BuildHasher, Hash, Hasher};
use hashbrown::raw::RawTable;
use iter::Iter;

pub(crate) struct Archetypes<R>
where
    R: Registry,
{
    raw_archetypes: RawTable<Archetype<R>>,
    hash_builder: ahash::RandomState,
}

impl<R> Archetypes<R>
where
    R: Registry,
{
    pub(crate) fn new() -> Self {
        Self {
            raw_archetypes: RawTable::new(),
            hash_builder: ahash::RandomState::new(),
        }
    }

    #[cfg(feature = "serde")]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            raw_archetypes: RawTable::with_capacity(capacity),
            hash_builder: ahash::RandomState::new(),
        }
    }

    fn make_hash(
        identifier: archetype::IdentifierRef<R>,
        hash_builder: &ahash::RandomState,
    ) -> u64 {
        let mut state = hash_builder.build_hasher();
        identifier.hash(&mut state);
        state.finish()
    }

    fn make_hasher(hash_builder: &ahash::RandomState) -> impl Fn(&Archetype<R>) -> u64 + '_ {
        move |archetype| {
            Self::make_hash(
                // SAFETY: The `IdentifierRef` obtained here does not live longer than the `archetype`.
                unsafe { archetype.identifier() },
                hash_builder,
            )
        }
    }

    fn equivalent_identifier(
        identifier: archetype::IdentifierRef<R>,
    ) -> impl Fn(&Archetype<R>) -> bool {
        move |archetype: &Archetype<R>| {
            (
                // SAFETY: The `IdentifierRef` obtained here does not live longer than the
                // `archetype`.
                unsafe { archetype.identifier() }
            ) == identifier
        }
    }

    pub(crate) fn get(&self, identifier: archetype::IdentifierRef<R>) -> Option<&Archetype<R>> {
        self.raw_archetypes.get(
            Self::make_hash(identifier, &self.hash_builder),
            Self::equivalent_identifier(identifier),
        )
    }

    pub(crate) fn get_mut_or_insert_new(
        &mut self,
        identifier_buffer: archetype::Identifier<R>,
    ) -> &mut Archetype<R> {
        let hash = Self::make_hash(
            // SAFETY: The `IdentifierRef` obtained here does not live longer than the
            // `identifier_buffer`.
            unsafe { identifier_buffer.as_ref() },
            &self.hash_builder,
        );

        match self.raw_archetypes.find(
            hash,
            Self::equivalent_identifier(
                // SAFETY: The `IdentifierRef` obtained here does not live longer than the
                // `identifier_buffer`.
                unsafe { identifier_buffer.as_ref() },
            ),
        ) {
            Some(archetype_bucket) =>
            // SAFETY: This reference to the archetype contained in this bucket is unique.
            unsafe { archetype_bucket.as_mut() },
            None => self.raw_archetypes.insert_entry(
                hash,
                Archetype::new(identifier_buffer),
                Self::make_hasher(&self.hash_builder),
            ),
        }
    }

    /// # Safety
    /// An archetype must be stored with the given `identifier`.
    pub(crate) unsafe fn get_unchecked_mut(
        &mut self,
        identifier: archetype::IdentifierRef<R>,
    ) -> &mut Archetype<R> {
        // SAFETY: The safety contract of this method guarantees that `get_mut()` will return a
        // `Some` value.
        unsafe {
            self.raw_archetypes
                .get_mut(
                    Self::make_hash(identifier, &self.hash_builder),
                    Self::equivalent_identifier(identifier),
                )
                .unwrap_unchecked()
        }
    }

    #[cfg(feature = "serde")]
    pub(crate) fn insert(&mut self, archetype: Archetype<R>) -> bool {
        let hash = Self::make_hash(
            // SAFETY: The `IdentifierRef` obtained here does not live longer than the `archetype`.
            unsafe { archetype.identifier() },
            &self.hash_builder,
        );
        if let Some(_existing_archetype) = self.raw_archetypes.get(
            hash,
            Self::equivalent_identifier(
                // SAFETY: The `IdentifierRef` obtained here does not live longer than the
                // `archetype`.
                unsafe { archetype.identifier() },
            ),
        ) {
            false
        } else {
            self.raw_archetypes
                .insert(hash, archetype, Self::make_hasher(&self.hash_builder));
            true
        }
    }

    pub(crate) fn iter(&self) -> Iter<R> {
        Iter::new(
            // SAFETY: The `Iter` containing this `RawIter` is guaranteed to not outlive `self`.
            unsafe { self.raw_archetypes.iter() },
        )
    }

    pub(crate) fn iter_mut(&mut self) -> IterMut<R> {
        IterMut::new(
            // SAFETY: The `IterMut` containing this `RawIter` is guaranteed to not outlive `self`.
            unsafe { self.raw_archetypes.iter() },
        )
    }

    /// # Safety
    /// `entity_allocator` must contain entries for each of the entities stored in the archetypes.
    pub(crate) unsafe fn clear(&mut self, entity_allocator: &mut entity::Allocator<R>) {
        for archetype in self.iter_mut() {
            // SAFETY: The `entity_allocator` is guaranteed to have an entry for each entity stored
            // in `archetype`.
            unsafe { archetype.clear(entity_allocator) };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{archetype, archetypes::Archetypes, registry};
    use alloc::vec;

    macro_rules! create_components {
        ($( $variants:ident ),*) => {
            $(
                struct $variants(f32);
            )*
        };
    }

    create_components!(
        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
    );

    type Registry =
        registry!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

    #[test]
    fn get_mut_or_insert_new_insertion() {
        let mut archetypes = Archetypes::<Registry>::new();
        let buffer = unsafe { archetype::Identifier::<Registry>::new(vec![1, 2, 3, 0]) };

        let archetype = archetypes.get_mut_or_insert_new(buffer);
    }

    #[test]
    fn get_mut_or_insert_new_already_inserted() {
        let mut archetypes = Archetypes::<Registry>::new();
        let buffer_a = unsafe { archetype::Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let buffer_b = unsafe { archetype::Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        archetypes.get_mut_or_insert_new(buffer_a);

        let archetype = archetypes.get_mut_or_insert_new(buffer_b);
    }
}
