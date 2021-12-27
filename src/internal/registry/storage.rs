use crate::{
    component::Component,
    internal::archetype,
    registry::{NullRegistry, Registry},
};
use alloc::vec::Vec;
use core::{
    any::TypeId,
    mem::{size_of, ManuallyDrop, MaybeUninit},
    ptr,
};
use hashbrown::HashMap;

pub trait RegistryStorage {
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize);

    unsafe fn create_component_map_for_key<R>(
        component_map: &mut HashMap<TypeId, usize>,
        index: usize,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where
        R: Registry;

    unsafe fn remove_component_row<R>(
        index: usize,
        removed_bytes: &mut Vec<u8>,
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where
        R: Registry;

    unsafe fn push_components_from_buffer_and_component<C, R>(
        buffer: *const u8,
        component: MaybeUninit<C>,
        components: &mut [(*mut u8, usize)],
        length: usize,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where
        C: Component,
        R: Registry;

    unsafe fn free_components<R>(
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where
        R: Registry;
}

impl RegistryStorage for NullRegistry {
    fn create_component_map(_component_map: &mut HashMap<TypeId, usize>, _index: usize) {}

    unsafe fn create_component_map_for_key<R>(
        _component_map: &mut HashMap<TypeId, usize>,
        _index: usize,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn remove_component_row<R>(
        _index: usize,
        _removed_bytes: &mut Vec<u8>,
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn push_components_from_buffer_and_component<C, R>(
        _buffer: *const u8,
        _component: MaybeUninit<C>,
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where
        C: Component,
        R: Registry,
    {
    }

    unsafe fn free_components<R>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where
        R: Registry,
    {
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

    unsafe fn create_component_map_for_key<R_>(
        component_map: &mut HashMap<TypeId, usize>,
        mut index: usize,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            component_map.insert(TypeId::of::<C>(), index);
            index += 1;
        }
        R::create_component_map_for_key(component_map, index, identifier_iter);
    }

    unsafe fn remove_component_row<R_>(
        index: usize,
        removed_bytes: &mut Vec<u8>,
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked(0);
            let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            ));

            removed_bytes.reserve(size_of::<C>());
            ptr::write(
                removed_bytes
                    .as_mut_ptr()
                    .add(removed_bytes.len())
                    .cast::<C>(),
                v.remove(index),
            );
            removed_bytes.set_len(removed_bytes.len() + size_of::<C>());

            components = components.get_unchecked(1..);
        }
        R::remove_component_row(index, removed_bytes, components, length, identifier_iter);
    }

    unsafe fn push_components_from_buffer_and_component<_C, _R>(
        mut buffer: *const u8,
        mut component: MaybeUninit<_C>,
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        mut identifier_iter: impl archetype::IdentifierIterator<_R>,
    ) where
        _C: Component,
        _R: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked_mut(0);

            if TypeId::of::<C>() == TypeId::of::<_C>() {
                // Consume the component. This is sound, since we won't ever read this
                // component again. This is because each component type is guaranteed to only
                // occur once within an Archetype's key.
                let mut v = ManuallyDrop::new(Vec::<_C>::from_raw_parts(
                    component_column.0.cast::<_C>(),
                    length,
                    component_column.1,
                ));
                v.push(component.assume_init());
                component = MaybeUninit::uninit();

                *component_column = (v.as_mut_ptr() as *mut u8, v.capacity());
            } else {
                let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
                    component_column.0.cast::<C>(),
                    length,
                    component_column.1,
                ));
                v.push(buffer.cast::<C>().read());
                buffer = buffer.add(size_of::<C>());

                *component_column = (v.as_mut_ptr() as *mut u8, v.capacity());
            }

            components = components.get_unchecked_mut(1..);
        }

        R::push_components_from_buffer_and_component(
            buffer,
            component,
            components,
            length,
            identifier_iter,
        );
    }

    unsafe fn free_components<R_>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked(0);
            let _ = Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            );
            components = components.get_unchecked(1..);
        }
        R::free_components(components, length, identifier_iter);
    }
}
