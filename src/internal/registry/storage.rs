use crate::{
    component::Component,
    registry::NullRegistry,
};
use alloc::vec::Vec;
use core::{
    any::TypeId,
    mem::size_of,
};
use hashbrown::HashMap;

pub trait RegistryStorage {
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize);

    unsafe fn create_component_map_for_key(
        component_map: &mut HashMap<TypeId, usize>,
        index: usize,
        key: &[u8],
        key_index: usize,
        bit: usize,
    );

    unsafe fn create_offset_map_for_key(
        offset_map: &mut HashMap<TypeId, isize>,
        offset: isize,
        key: &[u8],
        key_index: usize,
        bit: usize,
    );

    unsafe fn len_of_key(key: &[u8], key_index: usize, bit: usize) -> usize;

    unsafe fn free_components(
        components: &[(*mut u8, usize)],
        length: usize,
        key: &[u8],
        key_index: usize,
        bit: usize,
    );
}

impl RegistryStorage for NullRegistry {
    fn create_component_map(_component_map: &mut HashMap<TypeId, usize>, _index: usize) {}

    unsafe fn create_component_map_for_key(
        _component_map: &mut HashMap<TypeId, usize>,
        _index: usize,
        _key: &[u8],
        _key_index: usize,
        _bit: usize,
    ) {
    }

    unsafe fn create_offset_map_for_key(
        _offset_map: &mut HashMap<TypeId, isize>,
        _offset: isize,
        _key: &[u8],
        _key_index: usize,
        _bit: usize,
    ) {
    }

    unsafe fn len_of_key(_key: &[u8], _key_index: usize, _bit: usize) -> usize {
        0
    }

    unsafe fn free_components(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _key: &[u8],
        _key_index: usize,
        _bit: usize,
    ) {
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

    unsafe fn create_component_map_for_key(
        component_map: &mut HashMap<TypeId, usize>,
        mut index: usize,
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
            component_map.insert(TypeId::of::<C>(), index);
            index += 1;
        }
        R::create_component_map_for_key(component_map, index, key, new_key_index, new_bit);
    }

    unsafe fn create_offset_map_for_key(
        offset_map: &mut HashMap<TypeId, isize>,
        mut offset: isize,
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
            offset_map.insert(TypeId::of::<C>(), offset);
            offset += size_of::<C>() as isize;
        }
        R::create_offset_map_for_key(offset_map, offset, key, new_key_index, new_bit);
    }

    unsafe fn len_of_key(key: &[u8], key_index: usize, bit: usize) -> usize {
        let mut new_bit = bit + 1;
        let new_key_index = if new_bit >= 8 {
            new_bit &= 7;
            key_index + 1
        } else {
            key_index
        };

        (if key.get_unchecked(key_index) & (1 << (bit)) != 0 {
            1
        } else {
            0
        }) + R::len_of_key(key, new_key_index, new_bit)
    }

    unsafe fn free_components(
        mut components: &[(*mut u8, usize)],
        length: usize,
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
            let component_column = components.get_unchecked(0);
            let _ = Vec::<C>::from_raw_parts(
                component_column.0.cast::<C>(),
                length,
                component_column.1,
            );
            components = components.get_unchecked(1..);
        }
        R::free_components(
            components,
            length,
            key,
            new_key_index,
            new_bit,
        );
    }
}
