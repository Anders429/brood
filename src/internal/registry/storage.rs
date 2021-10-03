use crate::{
    component::Component,
    entities::Entities,
    entity::{Entity, EntityIdentifier},
    internal::archetype::Archetype,
    registry::NullRegistry,
};
use alloc::{boxed::Box, vec::Vec};
use core::{
    any::{Any, TypeId},
    marker::PhantomData,
};
use hashbrown::HashMap;
use unsafe_any::UnsafeAnyExt;

pub trait RegistryStorage {
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize);

    unsafe fn push<E1, E2>(
        entity: E1,
        entity_identifier: EntityIdentifier,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        canonical_entity: PhantomData<E2>,
    ) where
        E1: Entity,
        E2: Entity;

    unsafe fn extend<E1, E2, I>(
        entities: E1,
        entity_identifiers: I,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        canonical_entity: PhantomData<E2>,
    ) where
        E1: Entities,
        E2: Entity,
        I: Iterator<Item = EntityIdentifier>;
}

impl RegistryStorage for NullRegistry {
    fn create_component_map(_component_map: &mut HashMap<TypeId, usize>, _index: usize) {}

    unsafe fn push<E1, E2>(
        entity: E1,
        entity_identifier: EntityIdentifier,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        _index: usize,
        _bit: u8,
        _canonical_entity: PhantomData<E2>,
    ) where
        E1: Entity,
        E2: Entity,
    {
        unsafe {
            archetypes
                .entry(key)
                .or_insert(Box::new(Archetype::<E2>::new()))
                .downcast_mut_unchecked::<Archetype<E2>>()
                .push(entity, entity_identifier);
        }
    }

    unsafe fn extend<E1, E2, I>(
        entities: E1,
        entity_identifiers: I,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        _index: usize,
        _bit: u8,
        _canonical_entity: PhantomData<E2>,
    ) where
        E1: Entities,
        E2: Entity,
        I: Iterator<Item = EntityIdentifier>,
    {
        unsafe {
            archetypes
                .entry(key)
                .or_insert(Box::new(Archetype::<E2>::new()))
                .downcast_mut_unchecked::<Archetype<E2>>()
                .extend(entities, entity_identifiers);
        }
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
        entity_identifier: EntityIdentifier,
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
                entity_identifier,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
            );
        } else {
            R::push::<E1, E2>(
                entity,
                entity_identifier,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
            );
        }
    }

    unsafe fn extend<E1, E2, I>(
        entities: E1,
        entity_identifiers: I,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        _canonical_entity: PhantomData<E2>,
    ) where
        E1: Entities,
        E2: Entity,
        I: Iterator<Item = EntityIdentifier>,
    {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R::extend::<E1, (C, E2), I>(
                entities,
                entity_identifiers,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
            );
        } else {
            R::extend::<E1, E2, I>(
                entities,
                entity_identifiers,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
            );
        }
    }
}
