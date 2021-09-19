use crate::{component::Component, entities::NullEntities};
use alloc::vec::Vec;
use core::{any::TypeId, mem::ManuallyDrop};
use hashbrown::HashMap;

pub trait EntitiesStorage {
    unsafe fn into_buffer(self, buffer: *mut u8, component_map: &HashMap<TypeId, usize>);
    unsafe fn to_key(key: &mut [u8], component_map: &HashMap<TypeId, usize>);
}

impl EntitiesStorage for NullEntities {
    unsafe fn into_buffer(self, buffer: *mut u8, component_map: &HashMap<TypeId, usize>) {}
    unsafe fn to_key(key: &mut [u8], component_map: &HashMap<TypeId, usize>)
    {
    }
}

impl<C, E> EntitiesStorage for (Vec<C>, E)
where
    C: Component,
    E: EntitiesStorage,
{
    unsafe fn into_buffer(self, buffer: *mut u8, component_map: &HashMap<TypeId, usize>) {
        core::ptr::write(
            buffer
                .offset((*component_map.get(&TypeId::of::<C>()).unwrap() * 24) as isize)
                .cast::<Vec<C>>(),
            self.0,
        );
        E::into_buffer(self.1, buffer, component_map);
    }

    unsafe fn to_key(key: &mut [u8], component_map: &HashMap<TypeId, usize>)
    {
        let component_index = component_map.get(&TypeId::of::<C>()).unwrap();
        let index = component_index / 8;
        let bit = component_index % 8;

        *key.get_unchecked_mut(index) |= 1 << bit;

        E::to_key(key, component_map);
    }
}
