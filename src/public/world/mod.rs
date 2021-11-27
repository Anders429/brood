mod entry;
mod impl_debug;
mod impl_eq;
#[cfg(feature = "serde")]
mod impl_serde;

pub use entry::Entry;

use crate::{
    entities::{Entities, EntitiesIter},
    entity::{Entity, EntityIdentifier, NullEntity},
    internal::entity_allocator::EntityAllocator,
    query::{Filter, Query, Views},
    registry::Registry,
};
use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    any::{Any, TypeId},
    marker::PhantomData,
};
use hashbrown::HashMap;

pub struct World<R>
where
    R: Registry,
{
    archetypes: HashMap<Vec<u8>, Box<dyn Any>>,
    entity_allocator: EntityAllocator,

    registry: PhantomData<R>,
    component_map: HashMap<TypeId, usize>,
}

impl<R> World<R>
where
    R: Registry,
{
    fn from_raw_parts(
        archetypes: HashMap<Vec<u8>, Box<dyn Any>>,
        entity_allocator: EntityAllocator,
    ) -> Self {
        let mut component_map = HashMap::new();
        R::create_component_map(&mut component_map, 0);

        Self {
            archetypes,
            entity_allocator,

            registry: PhantomData,
            component_map,
        }
    }

    pub fn new() -> Self {
        Self::from_raw_parts(HashMap::new(), EntityAllocator::new())
    }

    pub fn push<E>(&mut self, entity: E)
    where
        E: Entity,
    {
        let mut key = vec![0; (R::LEN + 7) / 8];
        unsafe {
            E::to_key(&mut key, &self.component_map);
        }

        unsafe {
            R::push::<E, NullEntity>(
                entity,
                &mut self.entity_allocator,
                key,
                &mut self.archetypes,
                0,
                0,
                PhantomData,
            );
        }
    }

    pub fn extend<E>(&mut self, entities: EntitiesIter<E>)
    where
        E: Entities,
    {
        let mut key = vec![0; (R::LEN + 7) / 8];
        unsafe {
            E::to_key(&mut key, &self.component_map);
        }

        unsafe {
            R::extend::<E, NullEntity>(
                entities.entities,
                &mut self.entity_allocator,
                key,
                &mut self.archetypes,
                0,
                0,
                PhantomData,
            );
        }
    }

    pub fn query<'a, V, F>(&'a mut self, query: Query<'a, V, F>) -> V::Results
    where
        V: Views<'a>,
        F: Filter,
    {
        // 1. Construct key for filter.
        // 2. Iterate over all archetypes, finding keys that match the filter.
        // 3. Provide view to each matched archetype.
        //    This should return an iterator for each column requested in the view.
        // 4. Compile those slices into Vecs for each component in the view.
        // 5. Return the flattened Vecs, which will match the V::Results type definition.

        todo!()
    }

    pub fn entry(&mut self, entity_identifier: EntityIdentifier) -> Option<Entry<R>> {
        self.entity_allocator
            .get(entity_identifier)
            .map(|location| Entry::new(self, location))
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
