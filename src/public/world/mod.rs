mod entry;
mod impl_debug;
mod impl_default;
mod impl_eq;
#[cfg(feature = "serde")]
mod impl_serde;

pub use entry::Entry;

use crate::{
    entities::{Entities, EntitiesIter},
    entity::{Entity, EntityIdentifier},
    internal::{
        archetype::{self, Archetype},
        entity_allocator::EntityAllocator,
        query::FilterSeal,
    },
    query::{And, Filter, Views},
    registry::Registry,
};
use alloc::{vec, vec::Vec};
use core::{any::TypeId, iter};
use hashbrown::HashMap;

pub struct World<R>
where
    R: Registry,
{
    archetypes: HashMap<archetype::Identifier<R>, Archetype<R>>,
    entity_allocator: EntityAllocator<R>,

    component_map: HashMap<TypeId, usize>,
}

impl<R> World<R>
where
    R: Registry,
{
    fn from_raw_parts(
        archetypes: HashMap<archetype::Identifier<R>, Archetype<R>>,
        entity_allocator: EntityAllocator<R>,
    ) -> Self {
        let mut component_map = HashMap::new();
        R::create_component_map(&mut component_map, 0);

        Self {
            archetypes,
            entity_allocator,

            component_map,
        }
    }

    pub fn new() -> Self {
        Self::from_raw_parts(HashMap::new(), EntityAllocator::new())
    }

    pub fn push<E>(&mut self, entity: E) -> EntityIdentifier
    where
        E: Entity,
    {
        let mut key = vec![0; (R::LEN + 7) / 8];
        unsafe {
            E::to_key(&mut key, &self.component_map);
        }
        let identifier_buffer = unsafe { archetype::IdentifierBuffer::new(key) };

        unsafe {
            self.archetypes
                .entry(identifier_buffer.as_identifier())
                .or_insert(Archetype::<R>::new(identifier_buffer))
                .push(entity, &mut self.entity_allocator)
        }
    }

    // TODO: Figure out a way to remove the `must_use` attribute on the returned value.
    pub fn extend<E>(&mut self, entities: EntitiesIter<E>) -> impl Iterator<Item = EntityIdentifier>
    where
        E: Entities,
    {
        let mut key = vec![0; (R::LEN + 7) / 8];
        unsafe {
            E::to_key(&mut key, &self.component_map);
        }
        let identifier_buffer = unsafe { archetype::IdentifierBuffer::new(key) };

        unsafe {
            self.archetypes
                .entry(identifier_buffer.as_identifier())
                .or_insert(Archetype::<R>::new(identifier_buffer))
                .extend(entities, &mut self.entity_allocator)
        }
    }

    pub fn query<'a, V, F>(&'a mut self) -> iter::Flatten<vec::IntoIter<V::Results>>
    where
        V: Views<'a>,
        F: Filter,
    {
        self.archetypes
            .iter_mut()
            .filter(|(key, _archetype)| unsafe {
                And::<V, F>::filter(key.as_slice(), &self.component_map)
            })
            .map(|(_key, archetype)| archetype.view::<V>())
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
    }

    pub fn entry(&mut self, entity_identifier: EntityIdentifier) -> Option<Entry<R>> {
        self.entity_allocator
            .get(entity_identifier)
            .map(|location| Entry::new(self, location))
    }

    pub fn remove(&mut self, entity_identifier: EntityIdentifier) {
        // Get location of entity.
        if let Some(location) = self.entity_allocator.get(entity_identifier) {
            // Remove row from Archetype.
            unsafe {
                self.archetypes
                    .get_mut(&location.identifier)
                    .unwrap_unchecked()
                    .remove_row_unchecked(location.index, &mut self.entity_allocator);
            }
            // Free slot in EntityAllocator.
            unsafe {
                self.entity_allocator.free_unchecked(entity_identifier);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::{entities, entity, registry};
    use alloc::{borrow::ToOwned, string::String};

    #[test]
    fn push() {
        let mut world = World::<registry!(usize, bool, String, ())>::new();

        world.push(entity!(1_usize));
        world.push(entity!(true));
        world.push(entity!("foo".to_owned()));
    }

    #[test]
    fn extend() {
        let mut world = World::<registry!(usize, bool, String, ())>::new();

        world.extend(entities!((1_usize); 100));
        world.extend(entities!((true); 100));
        world.extend(entities!(("foo".to_owned()); 100));
        world.extend(entities!((2_usize, false, "bar".to_owned()); 100));
    }
}
