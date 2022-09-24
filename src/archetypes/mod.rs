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
    entity::{self, Entity},
    registry::{Canonical, Registry},
};
use core::{
    any::TypeId,
    hash::{BuildHasher, Hash, Hasher},
    hint::unreachable_unchecked,
};
use fnv::FnvBuildHasher;
use hashbrown::{raw::RawTable, HashMap};
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

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "serde")]
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
        self.raw_archetypes
            .shrink_to(0, Self::make_hasher(&self.hash_builder));
        for archetype in self.iter_mut() {
            archetype.shrink_to_fit();
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
