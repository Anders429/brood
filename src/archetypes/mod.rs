mod impl_debug;
mod impl_eq;
#[cfg(feature = "serde")]
mod impl_serde;
mod iter;
#[cfg(feature = "parallel")]
mod par_iter;

pub(crate) use iter::IterMut;
#[cfg(feature = "parallel")]
pub(crate) use par_iter::ParIterMut;

use crate::{archetype, archetype::Archetype, registry::Registry};
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
        move |archetype| Self::make_hash(unsafe { archetype.identifier() }, hash_builder)
    }

    fn equivalent_identifier(
        identifier: archetype::IdentifierRef<R>,
    ) -> impl Fn(&Archetype<R>) -> bool {
        move |archetype: &Archetype<R>| unsafe { archetype.identifier() } == identifier
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
        let hash = Self::make_hash(unsafe { identifier_buffer.as_ref() }, &self.hash_builder);

        match self.raw_archetypes.find(
            hash,
            Self::equivalent_identifier(unsafe { identifier_buffer.as_ref() }),
        ) {
            Some(archetype_bucket) => unsafe { archetype_bucket.as_mut() },
            None => self.raw_archetypes.insert_entry(
                hash,
                unsafe { Archetype::new(identifier_buffer) },
                Self::make_hasher(&self.hash_builder),
            ),
        }
    }

    pub(crate) unsafe fn get_unchecked_mut(
        &mut self,
        identifier: archetype::IdentifierRef<R>,
    ) -> &mut Archetype<R> {
        self.raw_archetypes
            .get_mut(
                Self::make_hash(identifier, &self.hash_builder),
                Self::equivalent_identifier(identifier),
            )
            .unwrap_unchecked()
    }

    #[cfg(feature = "serde")]
    pub(crate) fn insert(&mut self, archetype: Archetype<R>) -> bool {
        let hash = Self::make_hash(unsafe { archetype.identifier() }, &self.hash_builder);
        if let Some(_existing_archetype) = self.raw_archetypes.get(
            hash,
            Self::equivalent_identifier(unsafe { archetype.identifier() }),
        ) {
            false
        } else {
            self.raw_archetypes
                .insert(hash, archetype, Self::make_hasher(&self.hash_builder));
            true
        }
    }

    pub(crate) fn iter(&self) -> Iter<R> {
        Iter::new(unsafe { self.raw_archetypes.iter() })
    }

    pub(crate) fn iter_mut(&mut self) -> IterMut<R> {
        IterMut::new(unsafe { self.raw_archetypes.iter() })
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
