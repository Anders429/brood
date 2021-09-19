use crate::{
    component::Component,
    entity::{Entities, Entity},
    internal::archetype::Archetype,
    registry::{NullRegistry, Registry},
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

    unsafe fn push<E1, E2, R2>(
        entity: E1,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        canonical_entity: PhantomData<E2>,
        registry: PhantomData<R2>,
    ) where
        E1: Entity,
        E2: Entity,
        R2: Registry;

    unsafe fn extend<E1, E2, R2>(
        entities: E1,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        canonical_entity: PhantomData<E2>,
        registry: PhantomData<R2>,
    ) where
        E1: Entities,
        E2: Entity,
        R2: Registry;
}

impl RegistryStorage for NullRegistry {
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize) {}

    unsafe fn push<E1, E2, R2>(
        entity: E1,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        canonical_entity: PhantomData<E2>,
        registry: PhantomData<R2>,
    ) where
        E1: Entity,
        E2: Entity,
        R2: Registry,
    {
        unsafe {
            archetypes
                .entry(key)
                .or_insert(Box::new(Archetype::<E2>::new()))
                .downcast_mut_unchecked::<Archetype<E2>>()
                .push(entity);
        }
    }

    unsafe fn extend<E1, E2, R2>(
        entities: E1,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        canonical_entity: PhantomData<E2>,
        registry: PhantomData<R2>,
    ) where
        E1: Entities,
        E2: Entity,
        R2: Registry,
    {
        unsafe {
            archetypes
                .entry(key)
                .or_insert(Box::new(Archetype::<E2>::new()))
                .downcast_mut_unchecked::<Archetype<E2>>()
                .extend(entities);
        }
    }
}

impl<C, R1> RegistryStorage for (C, R1)
where
    C: Component,
    R1: RegistryStorage,
{
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize) {
        component_map.insert(TypeId::of::<C>(), index);
        R1::create_component_map(component_map, index + 1);
    }

    unsafe fn push<E1, E2, R2>(
        entity: E1,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        canonical_entity: PhantomData<E2>,
        registry: PhantomData<R2>,
    ) where
        E1: Entity,
        E2: Entity,
        R2: Registry,
    {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R1::push::<E1, (C, E2), R2>(
                entity,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
                PhantomData,
            );
        } else {
            R1::push::<E1, E2, R2>(
                entity,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
                PhantomData,
            );
        }
    }

    unsafe fn extend<E1, E2, R2>(
        entities: E1,
        key: Vec<u8>,
        archetypes: &mut HashMap<Vec<u8>, Box<dyn Any>>,
        index: usize,
        bit: u8,
        canonical_entity: PhantomData<E2>,
        registry: PhantomData<R2>,
    ) where
        E1: Entities,
        E2: Entity,
        R2: Registry,
    {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R1::extend::<E1, (C, E2), R2>(
                entities,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
                PhantomData,
            );
        } else {
            R1::extend::<E1, E2, R2>(
                entities,
                key,
                archetypes,
                new_index,
                new_bit,
                PhantomData,
                PhantomData,
            );
        }
    }
}
