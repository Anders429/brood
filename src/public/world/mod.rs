mod impl_debug;
mod impl_eq;
#[cfg(feature = "serde")]
mod impl_serde;

use crate::{
    entities::Entities,
    entity::{Entity, NullEntity},
    internal::entity_allocator::EntityAllocator,
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
    fn from_archetypes(archetypes: HashMap<Vec<u8>, Box<dyn Any>>) -> Self {
        let mut component_map = HashMap::new();
        R::create_component_map(&mut component_map, 0);

        Self {
            archetypes,
            entity_allocator: EntityAllocator::new(),

            registry: PhantomData,
            component_map,
        }
    }

    pub fn new() -> Self {
        Self::from_archetypes(HashMap::new())
    }

    pub fn push<E>(&mut self, entity: E)
    where
        E: Entity,
    {
        let mut key = vec![0; (R::LEN + 7) / 8];
        unsafe {
            E::to_key(&mut key, &self.component_map);
        }

        let entity_identifier = unsafe { self.entity_allocator.allocate(key.as_ptr()) };

        unsafe {
            R::push::<E, NullEntity>(
                entity,
                entity_identifier,
                key,
                &mut self.archetypes,
                0,
                0,
                PhantomData,
            );
        }
    }

    // TODO: This is currently not sound. It assumes entities has `Vec`s of the same length, but
    // there is no way that can be guaranteed.
    // `Entities` should be instead provided in a wrapper class that guarantees the length of the
    // component `Vec`s.
    pub fn extend<E>(&mut self, entities: E)
    where
        E: Entities,
    {
        let mut key = vec![0; (R::LEN + 7) / 8];
        unsafe {
            E::to_key(&mut key, &self.component_map);
        }

        let entity_identifiers = unsafe {
            self.entity_allocator
                .allocate_batch(key.as_ptr(), entities.component_len())
        };

        unsafe {
            R::extend::<E, NullEntity, _>(
                entities,
                entity_identifiers,
                key,
                &mut self.archetypes,
                0,
                0,
                PhantomData,
            );
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
