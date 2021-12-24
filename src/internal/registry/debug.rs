use crate::{
    component::Component,
    registry::{NullRegistry, Registry},
};
use alloc::vec::Vec;
use core::{
    any::type_name,
    fmt::{Debug, DebugList, DebugMap},
    mem::size_of,
};

pub trait RegistryDebug: Registry {
    unsafe fn extract_component_pointers(
        index: usize,
        components: &[(*mut u8, usize)],
        pointers: &mut Vec<*const u8>,
        key: &[u8],
        key_index: usize,
        bit: usize,
    );

    unsafe fn debug_components<'a, 'b>(
        pointers: &[*const u8],
        debug_map: &mut DebugMap<'a, 'b>,
        key: &[u8],
        key_index: usize,
        bit: usize,
    );

    unsafe fn debug_identifier<'a, 'b>(
        debug_list: &mut DebugList<'a, 'b>,
        key: &[u8],
        key_index: usize,
        bit: usize,
    );
}

impl RegistryDebug for NullRegistry {
    unsafe fn extract_component_pointers(
        _index: usize,
        _components: &[(*mut u8, usize)],
        _pointers: &mut Vec<*const u8>,
        _key: &[u8],
        _key_index: usize,
        _bit: usize,
    ) {
    }

    unsafe fn debug_components<'a, 'b>(
        _pointers: &[*const u8],
        _debug_map: &mut DebugMap<'a, 'b>,
        _key: &[u8],
        _key_index: usize,
        _bit: usize,
    ) {
    }

    unsafe fn debug_identifier<'a, 'b>(
        _debug_list: &mut DebugList<'a, 'b>,
        _key: &[u8],
        _key_index: usize,
        _bit: usize,
    ) {
    }
}

impl<C, R> RegistryDebug for (C, R)
where
    C: Component + Debug,
    R: RegistryDebug,
{
    unsafe fn extract_component_pointers(
        index: usize,
        mut components: &[(*mut u8, usize)],
        pointers: &mut Vec<*const u8>,
        key: &[u8],
        key_index: usize,
        bit: usize,
    ) {
        let mut new_bit = bit + 1;
        let new_key_index = if new_bit >= 8 {
            new_bit &= 7;
            key_index + 1
        } else {
            key_index
        };

        if key.get_unchecked(key_index) & (1 << (bit)) != 0 {
            pointers.push(components.get_unchecked(0).0.add(index * size_of::<C>()));
            components = components.get_unchecked(1..);
        }

        R::extract_component_pointers(index, components, pointers, key, new_key_index, new_bit);
    }

    unsafe fn debug_components<'a, 'b>(
        mut pointers: &[*const u8],
        debug_map: &mut DebugMap<'a, 'b>,
        key: &[u8],
        key_index: usize,
        bit: usize,
    ) {
        let mut new_bit = bit + 1;
        let new_key_index = if new_bit >= 8 {
            new_bit &= 7;
            key_index + 1
        } else {
            key_index
        };

        if key.get_unchecked(key_index) & (1 << (bit)) != 0 {
            debug_map.entry(&type_name::<C>(), &*pointers.get_unchecked(0).cast::<C>());
            pointers = pointers.get_unchecked(1..);
        }

        R::debug_components(pointers, debug_map, key, new_key_index, new_bit);
    }

    unsafe fn debug_identifier<'a, 'b>(
        debug_list: &mut DebugList<'a, 'b>,
        key: &[u8],
        key_index: usize,
        bit: usize,
    ) {
        let mut new_bit = bit + 1;
        let new_key_index = if new_bit >= 8 {
            new_bit &= 7;
            key_index + 1
        } else {
            key_index
        };

        if key.get_unchecked(key_index) & (1 << (bit)) != 0 {
            debug_list.entry(&type_name::<C>());
        }

        R::debug_identifier(debug_list, key, new_key_index, new_bit);
    }
}
