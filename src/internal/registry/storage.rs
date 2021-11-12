use crate::{
    component::Component,
    entities::Entities,
    entity::Entity,
    internal::{archetype::Archetype, entity_allocator::EntityAllocator},
    registry::NullRegistry,
};
use alloc::{boxed::Box, vec::Vec};
use core::{
    any::{Any, TypeId},
    marker::PhantomData,
    ptr,
};
use hashbrown::HashMap;
use unsafe_any::UnsafeAnyExt;

pub trait RegistryStorage {
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize);

    unsafe fn push<E1, E2>(
        entity: E1,
        entity_allocator: &mut EntityAllocator,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        canonical_entity: PhantomData<E2>,
    ) where
        E1: Entity,
        E2: Entity;

    unsafe fn extend<E1, E2>(
        entities: E1,
        entity_allocator: &mut EntityAllocator,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        canonical_entity: PhantomData<E2>,
    ) where
        E1: Entities,
        E2: Entity;
}

impl RegistryStorage for NullRegistry {
    fn create_component_map(_component_map: &mut HashMap<TypeId, usize>, _index: usize) {}

    unsafe fn push<E1, E2>(
        entity: E1,
        entity_allocator: &mut EntityAllocator,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        _index: usize,
        _bit: u8,
        _canonical_entity: PhantomData<E2>,
    ) where
        E1: Entity,
        E2: Entity,
    {
        let archetype_entry = archetypes.entry(key);

        let entity_identifier = entity_allocator.allocate(ptr::NonNull::new_unchecked(archetype_entry.key().as_ptr() as *mut u8));

        archetype_entry
            .or_insert(Box::new(Archetype::<E2>::new()))
            .downcast_mut_unchecked::<Archetype<E2>>()
            .push(entity, entity_identifier);
    }

    unsafe fn extend<E1, E2>(
        entities: E1,
        entity_allocator: &mut EntityAllocator,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        _index: usize,
        _bit: u8,
        _canonical_entity: PhantomData<E2>,
    ) where
        E1: Entities,
        E2: Entity,
    {
        let archetype_entry = archetypes.entry(key);

        let entity_identifiers = entity_allocator
            .allocate_batch(ptr::NonNull::new_unchecked(archetype_entry.key().as_ptr() as *mut u8), entities.component_len());

        archetype_entry
            .or_insert(Box::new(Archetype::<E2>::new()))
            .downcast_mut_unchecked::<Archetype<E2>>()
            .extend(entities, entity_identifiers);
    }
}

impl<C, R> RegistryStorage for (C, R)
where
    C: Component,
    R: RegistryStorage,
{
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize) {
        component_map.insert(TypeId::of::<C>(), index);
        R::create_component_map(component_map, index + 1);
    }

    unsafe fn push<E1, E2>(
        entity: E1,
        entity_allocator: &mut EntityAllocator,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        _canonical_entity: PhantomData<E2>,
    ) where
        E1: Entity,
        E2: Entity,
    {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R::push::<E1, (C, E2)>(
                entity,
                entity_allocator,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
            );
        } else {
            R::push::<E1, E2>(
                entity,
                entity_allocator,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
            );
        }
    }

    unsafe fn extend<E1, E2>(
        entities: E1,
        entity_allocator: &mut EntityAllocator,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        _canonical_entity: PhantomData<E2>,
    ) where
        E1: Entities,
        E2: Entity,
    {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R::extend::<E1, (C, E2)>(
                entities,
                entity_allocator,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
            );
        } else {
            R::extend::<E1, E2>(
                entities,
                entity_allocator,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
            );
        }
    }
}
