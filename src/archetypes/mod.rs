mod impl_debug;
mod impl_eq;
#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;
mod iter;
#[cfg(feature = "rayon")]
mod par_iter;

#[cfg(feature = "serde")]
pub(crate) use impl_serde::DeserializeArchetypes;
pub(crate) use iter::IterMut;
#[cfg(feature = "rayon")]
pub(crate) use par_iter::ParIterMut;

use crate::{
    archetype,
    archetype::Archetype,
    entity::{
        self,
        Entity,
    },
    registry,
    registry::{
        Canonical,
        Registry,
    },
};
use alloc::vec::Vec;
use by_address::ByThinAddress;
use core::{
    any::TypeId,
    hash::{
        BuildHasher,
        Hash,
        Hasher,
    },
    hint::unreachable_unchecked,
};
use fnv::FnvBuildHasher;
use hashbrown::{
    raw::RawTable,
    HashMap,
    HashSet,
};
use iter::Iter;

pub(crate) struct Archetypes<R>
where
    R: Registry,
{
    raw_archetypes: RawTable<Archetype<R>>,
    hash_builder: FnvBuildHasher,

    type_id_lookup: HashMap<TypeId, archetype::IdentifierRef<R>, FnvBuildHasher>,
}

impl<R> Archetypes<R>
where
    R: Registry,
{
    pub(crate) fn new() -> Self {
        Self {
            raw_archetypes: RawTable::new(),
            hash_builder: FnvBuildHasher::default(),

            type_id_lookup: HashMap::default(),
        }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            raw_archetypes: RawTable::with_capacity(capacity),
            hash_builder: FnvBuildHasher::default(),

            type_id_lookup: HashMap::with_capacity_and_hasher(capacity, FnvBuildHasher::default()),
        }
    }

    fn make_hash(identifier: archetype::IdentifierRef<R>, hash_builder: &FnvBuildHasher) -> u64 {
        let mut state = hash_builder.build_hasher();
        identifier.hash(&mut state);
        state.finish()
    }

    fn make_hasher(hash_builder: &FnvBuildHasher) -> impl Fn(&Archetype<R>) -> u64 + '_ {
        move |archetype| {
            Self::make_hash(
                // SAFETY: The `IdentifierRef` obtained here does not live longer than the
                // `archetype`.
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

    /// Returns a reference to the `Archetype` identified by the given identifier.
    ///
    /// If no `Archetype` exists for the identifier, `None` is returned.
    pub(crate) fn get(&self, identifier: archetype::IdentifierRef<R>) -> Option<&Archetype<R>> {
        self.raw_archetypes.get(
            Self::make_hash(identifier, &self.hash_builder),
            Self::equivalent_identifier(identifier),
        )
    }

    /// Returns a mutable reference to the `Archetype` identified by the given identifier.
    ///
    /// If no `Archetype` exists for the identifier, `None` is returned.
    pub(crate) fn get_mut(
        &mut self,
        identifier: archetype::IdentifierRef<R>,
    ) -> Option<&mut Archetype<R>> {
        self.raw_archetypes.get_mut(
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
    /// `component_map` must contain an entry for each component in the entity `E`. Each entry must
    /// correspond to its component's location in the registry `R`.
    pub(crate) unsafe fn get_mut_or_insert_new_for_entity<E, P>(&mut self) -> &mut Archetype<R>
    where
        E: Entity,
        R: Canonical<E, P>,
    {
        // Lookup the archetype identifier.
        if let Some(identifier) = self.type_id_lookup.get(&TypeId::of::<E>()) {
            let hash = Self::make_hash(*identifier, &self.hash_builder);

            match self
                .raw_archetypes
                .find(hash, Self::equivalent_identifier(*identifier))
            {
                // SAFETY: This reference to the archetype contained in this bucket is unique.
                Some(archetype_bucket) => unsafe { archetype_bucket.as_mut() },
                // SAFETY: If the type has an entry in `self.type_id_lookup`, then it will
                // invariantly have an archetype stored.
                None => unsafe { unreachable_unchecked() },
            }
        } else {
            let identifier = R::create_archetype_identifier();

            let hash = Self::make_hash(
                // SAFETY: The `IdentifierRef` obtained here does not live longer than the
                // `identifier_buffer`.
                unsafe { identifier.as_ref() },
                &self.hash_builder,
            );

            if let Some(archetype_bucket) = self.raw_archetypes.find(
                hash,
                Self::equivalent_identifier(
                    // SAFETY: The `IdentifierRef` obtained here does not live longer than the
                    // `identifier_buffer`.
                    unsafe { identifier.as_ref() },
                ),
            ) {
                // SAFETY: This reference to the archetype contained in this bucket is unique.
                unsafe { archetype_bucket.as_mut() }
            } else {
                self.type_id_lookup.insert(
                    TypeId::of::<E>(),
                    // SAFETY: The `IdentifierRef` obtained here does not live longer than the
                    // `identifier_buffer`.
                    unsafe { identifier.as_ref() },
                );
                self.raw_archetypes.insert_entry(
                    hash,
                    Archetype::new(identifier),
                    Self::make_hasher(&self.hash_builder),
                )
            }
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

    pub(crate) fn insert(&mut self, archetype: Archetype<R>) -> Result<(), Archetype<R>> {
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
            Err(archetype)
        } else {
            self.raw_archetypes
                .insert(hash, archetype, Self::make_hasher(&self.hash_builder));
            Ok(())
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

    /// Decrease the allocated capacity to the smallest amount required for the stored data.
    ///
    /// This may not decrease to the most optimal value, as the shrinking is dependent on the
    /// allocator.
    pub(crate) fn shrink_to_fit(&mut self) {
        let mut identifiers_to_erase = HashSet::with_hasher(FnvBuildHasher::default());
        let mut archetypes_to_erase = Vec::new();
        // SAFETY: The resulting `RawIter` is guaranteed to not outlive `self.raw_archetypes`.
        for archetype_bucket in unsafe { self.raw_archetypes.iter() } {
            // SAFETY: The reference to the archetype stored in this bucket is guaranteed to be
            // unique.
            let archetype = unsafe { archetype_bucket.as_mut() };
            // Only retain non-empty archetypes.
            if archetype.is_empty() {
                identifiers_to_erase.insert(
                    // SAFETY: This identifier will outlive its archetype, since the archetypes are
                    // deleted after the identifiers are used.
                    unsafe { archetype.identifier() },
                );
                archetypes_to_erase.push(archetype_bucket);
            } else {
                archetype.shrink_to_fit();
            }
        }

        // Removing from `self.type_id_lookup` guarantees that the invariant that any entry in
        // `type_id_lookup` corresponds to a valid archetype is still upheld.
        for type_id in self
            .type_id_lookup
            .iter()
            .filter_map(|(&type_id, identifier)| {
                if identifiers_to_erase.contains(identifier) {
                    Some(type_id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
        {
            self.type_id_lookup.remove(&type_id);
        }

        for archetype_bucket in archetypes_to_erase {
            // SAFETY: `archetype` is not used again after it is dropped from the table.
            unsafe {
                self.raw_archetypes.erase(archetype_bucket);
            }
        }

        self.raw_archetypes
            .shrink_to(0, Self::make_hasher(&self.hash_builder));
    }
}

impl<R> Archetypes<R>
where
    R: registry::Clone,
{
    /// Clone the archetypes.
    ///
    /// Returns both the new archetypes and a map from the source archetype identifiers to their
    /// equivalent archetype identifiers in the new archetypes.
    ///
    /// # Safety
    /// The returned `HashMap` must outlive both the original and cloned archetypes.
    pub(crate) unsafe fn clone(
        &self,
    ) -> (
        Self,
        HashMap<ByThinAddress<&'static [u8]>, archetype::IdentifierRef<R>, FnvBuildHasher>,
    ) {
        let mut identifier_map =
            HashMap::with_capacity_and_hasher(self.raw_archetypes.len(), FnvBuildHasher::default());
        let mut cloned_archetypes = Self::with_capacity(self.raw_archetypes.len());

        for archetype in self.iter() {
            let cloned_archetype = archetype.clone();
            identifier_map.insert(
                // SAFETY: This slice will outlive the original archetype by the safety contract of
                // this method.
                ByThinAddress(unsafe { archetype.identifier().as_slice() }),
                // SAFETY: This `IdentifierRef` will outlive the cloned archetype by the safety
                // contract of this method.
                unsafe { cloned_archetype.identifier() },
            );
            #[allow(unused_must_use)]
            {
                cloned_archetypes.insert(cloned_archetype);
            }
        }

        for (&type_id, identifier) in self.type_id_lookup.iter() {
            cloned_archetypes.type_id_lookup.insert(
                type_id,
                // SAFETY: Each identifier in `self.type_id_lookup` is guaranteed to be found in
                // `identifier_map`.
                *unsafe {
                    identifier_map
                        .get(&ByThinAddress(identifier.as_slice()))
                        .unwrap_unchecked()
                },
            );
        }

        (cloned_archetypes, identifier_map)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        archetype,
        archetypes::Archetypes,
        Registry,
    };
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
        Registry!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

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
