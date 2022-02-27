use crate::{
    archetype,
    component::Component,
    registry::{Null, Registry},
};
use alloc::vec::Vec;
use core::{
    any::{type_name, TypeId},
    fmt::DebugList,
    marker::PhantomData,
    mem::{drop, size_of, ManuallyDrop, MaybeUninit},
    ptr,
};
use hashbrown::HashMap;

pub trait Storage {
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize);

    unsafe fn create_component_map_for_key<R>(
        component_map: &mut HashMap<TypeId, usize>,
        index: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    unsafe fn new_components_with_capacity<R>(
        components: &mut Vec<(*mut u8, usize)>,
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    unsafe fn size_of_components_for_identifier<R>(
        identifier_iter: archetype::identifier::Iter<R>,
    ) -> usize
    where
        R: Registry;

    unsafe fn remove_component_row<R>(
        index: usize,
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    unsafe fn pop_component_row<R>(
        index: usize,
        buffer: *mut u8,
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    unsafe fn push_components_from_buffer_and_component<C, R>(
        buffer: *const u8,
        component: MaybeUninit<C>,
        components: &mut [(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        C: Component,
        R: Registry;

    unsafe fn push_components_from_buffer_skipping_component<C, R>(
        buffer: *const u8,
        component: PhantomData<C>,
        components: &mut [(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        C: Component,
        R: Registry;

    unsafe fn free_components<R>(
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    unsafe fn try_free_components<R>(
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    unsafe fn clear_components<R>(
        components: &mut [(*mut u8, usize)],
        length: usize,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;

    unsafe fn debug_identifier<R>(
        debug_list: &mut DebugList,
        identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry;
}

impl Storage for Null {
    fn create_component_map(_component_map: &mut HashMap<TypeId, usize>, _index: usize) {}

    unsafe fn create_component_map_for_key<R>(
        _component_map: &mut HashMap<TypeId, usize>,
        _index: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn new_components_with_capacity<R>(
        _components: &mut Vec<(*mut u8, usize)>,
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn size_of_components_for_identifier<R>(
        _identifier_iter: archetype::identifier::Iter<R>,
    ) -> usize
    where
        R: Registry,
    {
        0
    }

    unsafe fn remove_component_row<R>(
        _index: usize,
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn pop_component_row<R>(
        _index: usize,
        _buffer: *mut u8,
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn push_components_from_buffer_and_component<C, R>(
        _buffer: *const u8,
        _component: MaybeUninit<C>,
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        C: Component,
        R: Registry,
    {
    }

    unsafe fn push_components_from_buffer_skipping_component<C, R>(
        _buffer: *const u8,
        _component: PhantomData<C>,
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        C: Component,
        R: Registry,
    {
    }

    unsafe fn free_components<R>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn try_free_components<R>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn clear_components<R>(
        _components: &mut [(*mut u8, usize)],
        _length: usize,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }

    unsafe fn debug_identifier<R>(
        _debug_list: &mut DebugList,
        _identifier_iter: archetype::identifier::Iter<R>,
    ) where
        R: Registry,
    {
    }
}

impl<C, R> Storage for (C, R)
where
    C: Component,
    R: Storage,
{
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize) {
        component_map.insert(TypeId::of::<C>(), index);
        R::create_component_map(component_map, index + 1);
    }

    unsafe fn create_component_map_for_key<R_>(
        component_map: &mut HashMap<TypeId, usize>,
        mut index: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            component_map.insert(TypeId::of::<C>(), index);
            index += 1;
        }
        R::create_component_map_for_key(component_map, index, identifier_iter);
    }

    unsafe fn new_components_with_capacity<R_>(
        components: &mut Vec<(*mut u8, usize)>,
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let mut v = ManuallyDrop::new(Vec::<C>::with_capacity(length));
            components.push((v.as_mut_ptr().cast::<u8>(), v.capacity()));
        }

        R::new_components_with_capacity(components, length, identifier_iter);
    }

    unsafe fn size_of_components_for_identifier<R_>(
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) -> usize
    where
        R_: Registry,
    {
        (usize::from(identifier_iter.next().unwrap_unchecked()) * size_of::<C>())
            + R::size_of_components_for_identifier(identifier_iter)
    }

    unsafe fn remove_component_row<R_>(
        index: usize,
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
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
            v.swap_remove(index);

            components = components.get_unchecked(1..);
        }
        R::remove_component_row(index, components, length, identifier_iter);
    }

    unsafe fn pop_component_row<R_>(
        index: usize,
        mut buffer: *mut u8,
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
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

            ptr::write_unaligned(buffer.cast::<C>(), v.swap_remove(index));
            buffer = buffer.add(size_of::<C>());

            components = components.get_unchecked(1..);
        }
        R::pop_component_row(index, buffer, components, length, identifier_iter);
    }

    unsafe fn push_components_from_buffer_and_component<C_, R_>(
        mut buffer: *const u8,
        mut component: MaybeUninit<C_>,
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        C_: Component,
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked_mut(0);

            if TypeId::of::<C>() == TypeId::of::<C_>() {
                // Consume the component. This is sound, since we won't ever read this
                // component again. This is because each component type is guaranteed to only
                // occur once within an Archetype's identifier.
                let mut v = ManuallyDrop::new(Vec::<C_>::from_raw_parts(
                    component_column.0.cast::<C_>(),
                    length,
                    component_column.1,
                ));
                v.push(component.assume_init());
                component = MaybeUninit::uninit();

                *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
            } else {
                let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
                    component_column.0.cast::<C>(),
                    length,
                    component_column.1,
                ));
                v.push(buffer.cast::<C>().read_unaligned());
                buffer = buffer.add(size_of::<C>());

                *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());
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

    unsafe fn push_components_from_buffer_skipping_component<C_, R_>(
        mut buffer: *const u8,
        component: PhantomData<C_>,
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        C_: Component,
        R_: Registry,
    {
        if TypeId::of::<C>() == TypeId::of::<C_>() {
            // Skip this component in the buffer.
            buffer = buffer.add(size_of::<C>());
            // Pop the next identifier bit. We don't need to look at it, since the component is
            // being skipped anyway.
            identifier_iter.next();
        } else if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked_mut(0);
            let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            ));
            v.push(buffer.cast::<C>().read_unaligned());
            *component_column = (v.as_mut_ptr().cast::<u8>(), v.capacity());

            components = components.get_unchecked_mut(1..);
            buffer = buffer.add(size_of::<C>());
        }

        R::push_components_from_buffer_skipping_component(
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
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked(0);
            drop(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            ));
            components = components.get_unchecked(1..);
        }
        R::free_components(components, length, identifier_iter);
    }

    unsafe fn try_free_components<R_>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = match components.get(0) {
                Some(component_column) => component_column,
                None => {
                    return;
                }
            };
            drop(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            ));
            components = match components.get(1..) {
                Some(components) => components,
                None => {
                    return;
                }
            };
        }
        R::try_free_components(components, length, identifier_iter);
    }

    unsafe fn clear_components<R_>(
        mut components: &mut [(*mut u8, usize)],
        length: usize,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column = components.get_unchecked_mut(0);
            let mut v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            ));
            v.clear();
            components = components.get_unchecked_mut(1..);
        }

        R::clear_components(components, length, identifier_iter);
    }

    unsafe fn debug_identifier<R_>(
        debug_list: &mut DebugList,
        mut identifier_iter: archetype::identifier::Iter<R_>,
    ) where
        R_: Registry,
    {
        if identifier_iter.next().unwrap_unchecked() {
            debug_list.entry(&type_name::<C>());
        }

        R::debug_identifier(debug_list, identifier_iter);
    }
}
