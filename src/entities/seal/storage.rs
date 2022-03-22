use crate::{component::Component, entities::Null};
use alloc::vec::Vec;
use core::{any::TypeId, mem::ManuallyDrop};
use hashbrown::HashMap;

pub trait Storage {
    unsafe fn extend_components(
        self,
        component_map: &HashMap<TypeId, usize>,
        components: &mut [(*mut u8, usize)],
        length: usize,
    );

    unsafe fn to_identifier(identifier: &mut [u8], component_map: &HashMap<TypeId, usize>);
}

impl Storage for Null {
    unsafe fn extend_components(
        self,
        _component_map: &HashMap<TypeId, usize>,
        _components: &mut [(*mut u8, usize)],
        _length: usize,
    ) {
    }

    unsafe fn to_identifier(_identifier: &mut [u8], _component_map: &HashMap<TypeId, usize>) {}
}

impl<C, E> Storage for (Vec<C>, E)
where
    C: Component,
    E: Storage,
{
    unsafe fn extend_components(
        self,
        component_map: &HashMap<TypeId, usize>,
        components: &mut [(*mut u8, usize)],
        length: usize,
    ) {
        let component_column =
            components.get_unchecked_mut(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked());
        if length == 0 {
            let mut v = ManuallyDrop::new(self.0);
            *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
        } else {
            let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            ));
            v.extend(self.0);
            *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
        }
        E::extend_components(self.1, component_map, components, length);
    }

    unsafe fn to_identifier(identifier: &mut [u8], component_map: &HashMap<TypeId, usize>) {
        let component_index = component_map.get(&TypeId::of::<C>()).unwrap();
        let index = component_index / 8;
        let bit = component_index % 8;

        *identifier.get_unchecked_mut(index) |= 1 << bit;

        E::to_identifier(identifier, component_map);
    }
}
