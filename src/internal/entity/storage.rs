use crate::{
    component::Component,
    entity::{NullEntities, NullEntity},
    registry::Registry,
};
use alloc::vec::Vec;
use core::{
    any::TypeId,
    mem::{size_of, ManuallyDrop},
};
use hashbrown::HashMap;

pub trait EntityStorage {
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize);
    fn create_offset_map(offset_map: &mut HashMap<TypeId, isize>, offset: isize);
    unsafe fn into_buffer(self, buffer: *mut u8, offset_map: &HashMap<TypeId, isize>);
    unsafe fn push_components_from_buffer(
        buffer: *const u8,
        components: &mut [(*mut u8, usize)],
        length: usize,
    );
    unsafe fn extend_components_from_buffer(
        buffer: *const u8,
        components: &mut [(*mut u8, usize)],
        length: usize,
    );
    unsafe fn free_components(components: &[(*mut u8, usize)], length: usize);
    unsafe fn to_key<R>(key: &mut [u8; (R::LEN + 7) / 8], component_map: &HashMap<TypeId, usize>)
    where
        R: Registry;
}

impl EntityStorage for NullEntity {
    fn create_component_map(_component_map: &mut HashMap<TypeId, usize>, _index: usize) {}
    fn create_offset_map(_offset_map: &mut HashMap<TypeId, isize>, _offset: isize) {}
    unsafe fn into_buffer(self, _buffer: *mut u8, _offset_map: &HashMap<TypeId, isize>) {}
    unsafe fn push_components_from_buffer(
        _buffer: *const u8,
        _components: &mut [(*mut u8, usize)],
        _length: usize,
    ) {
    }
    unsafe fn extend_components_from_buffer(
        buffer: *const u8,
        components: &mut [(*mut u8, usize)],
        length: usize,
    ) {
    }
    unsafe fn free_components(components: &[(*mut u8, usize)], length: usize) {}
    unsafe fn to_key<R>(key: &mut [u8; (R::LEN + 7) / 8], component_map: &HashMap<TypeId, usize>)
    where
        R: Registry,
    {
    }
}

impl<C, E> EntityStorage for (C, E)
where
    C: Component,
    E: EntityStorage,
{
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize) {
        component_map.insert(TypeId::of::<C>(), index);
        E::create_component_map(component_map, index + 1);
    }

    fn create_offset_map(offset_map: &mut HashMap<TypeId, isize>, offset: isize) {
        offset_map.insert(TypeId::of::<C>(), offset);
        E::create_offset_map(offset_map, offset + size_of::<C>() as isize);
    }

    unsafe fn into_buffer(self, buffer: *mut u8, offset_map: &HashMap<TypeId, isize>) {
        core::ptr::write(
            buffer
                .offset(*offset_map.get(&TypeId::of::<C>()).unwrap())
                .cast::<C>(),
            self.0,
        );
        E::into_buffer(self.1, buffer, offset_map);
    }

    unsafe fn push_components_from_buffer(
        buffer: *const u8,
        components: &mut [(*mut u8, usize)],
        length: usize,
    ) {
        let component_column = components.get_unchecked_mut(0);
        let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
            component_column.0.cast::<C>(),
            length,
            component_column.1,
        ));
        v.push(buffer.cast::<C>().read());
        *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
        E::push_components_from_buffer(
            buffer.offset(size_of::<C>() as isize),
            components.get_unchecked_mut(1..),
            length,
        );
    }

    unsafe fn extend_components_from_buffer(
        buffer: *const u8,
        components: &mut [(*mut u8, usize)],
        length: usize,
    ) {
        let component_column = components.get_unchecked_mut(0);
        let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
            component_column.0.cast::<C>(),
            length,
            component_column.1,
        ));
        v.extend(buffer.cast::<Vec<C>>().read());
        *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
        E::extend_components_from_buffer(
            buffer.offset(24),
            components.get_unchecked_mut(1..),
            length,
        );
    }

    unsafe fn free_components(components: &[(*mut u8, usize)], length: usize) {
        let component_column = components.get_unchecked(0);
        let _ =
            Vec::<C>::from_raw_parts(component_column.0.cast::<C>(), length, component_column.1);
        E::free_components(components.get_unchecked(1..), length);
    }

    unsafe fn to_key<R>(key: &mut [u8; (R::LEN + 7) / 8], component_map: &HashMap<TypeId, usize>)
    where
        R: Registry,
    {
        let component_index = component_map.get(&TypeId::of::<C>()).unwrap();
        let index = component_index / 8;
        let bit = component_index % 8;

        *key.get_unchecked_mut(index) |= 1 << bit;

        E::to_key::<R>(key, component_map);
    }
}

pub trait EntitiesStorage {
    unsafe fn into_buffer(self, buffer: *mut u8, component_map: &HashMap<TypeId, usize>);
    unsafe fn to_key<R>(key: &mut [u8; (R::LEN + 7) / 8], component_map: &HashMap<TypeId, usize>)
    where
        R: Registry;
}

impl EntitiesStorage for NullEntities {
    unsafe fn into_buffer(self, buffer: *mut u8, component_map: &HashMap<TypeId, usize>) {}
    unsafe fn to_key<R>(key: &mut [u8; (R::LEN + 7) / 8], component_map: &HashMap<TypeId, usize>)
    where
        R: Registry,
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

    unsafe fn to_key<R>(key: &mut [u8; (R::LEN + 7) / 8], component_map: &HashMap<TypeId, usize>)
    where
        R: Registry,
    {
        let component_index = component_map.get(&TypeId::of::<C>()).unwrap();
        let index = component_index / 8;
        let bit = component_index % 8;

        *key.get_unchecked_mut(index) |= 1 << bit;

        E::to_key::<R>(key, component_map);
    }
}
