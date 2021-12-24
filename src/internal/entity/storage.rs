use crate::{component::Component, entity::NullEntity};
use alloc::vec::Vec;
use core::{
    any::TypeId,
    mem::ManuallyDrop,
};
use hashbrown::HashMap;

pub trait EntityStorage {
    unsafe fn push_components(
        self,
        component_map: &HashMap<TypeId, usize>,
        components: &mut [(*mut u8, usize)],
        length: usize,
    );

    unsafe fn to_key(key: &mut [u8], component_map: &HashMap<TypeId, usize>);
}

impl EntityStorage for NullEntity {
    unsafe fn push_components(
        self,
        _component_map: &HashMap<TypeId, usize>,
        _components: &mut [(*mut u8, usize)],
        _length: usize,
    ) {
    }

    unsafe fn to_key(_key: &mut [u8], _component_map: &HashMap<TypeId, usize>) {}
}

impl<C, E> EntityStorage for (C, E)
where
    C: Component,
    E: EntityStorage,
{
    unsafe fn push_components(
        self,
        component_map: &HashMap<TypeId, usize>,
        components: &mut [(*mut u8, usize)],
        length: usize,
    ) {
        let component_column =
            components.get_unchecked_mut(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked());
        let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
            component_column.0.cast::<C>(),
            length,
            component_column.1,
        ));
        v.push(self.0);
        *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
        E::push_components(self.1, component_map, components, length);
    }

    unsafe fn to_key(key: &mut [u8], component_map: &HashMap<TypeId, usize>) {
        let component_index = component_map.get(&TypeId::of::<C>()).unwrap();
        let index = component_index / 8;
        let bit = component_index % 8;

        *key.get_unchecked_mut(index) |= 1 << bit;

        E::to_key(key, component_map);
    }
}
