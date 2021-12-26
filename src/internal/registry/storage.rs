use crate::{component::Component, internal::archetype, registry::{NullRegistry, Registry}};
use alloc::vec::Vec;
use core::any::TypeId;
use hashbrown::HashMap;

pub trait RegistryStorage {
    fn create_component_map(component_map: &mut HashMap<TypeId, usize>, index: usize);

    unsafe fn create_component_map_for_key<R>(
        component_map: &mut HashMap<TypeId, usize>,
        index: usize,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where R: Registry;

    unsafe fn free_components<R>(
        components: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where R: Registry;
}

impl RegistryStorage for NullRegistry {
    fn create_component_map(_component_map: &mut HashMap<TypeId, usize>, _index: usize) {}

    unsafe fn create_component_map_for_key<R>(
        _component_map: &mut HashMap<TypeId, usize>,
        _index: usize,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where R: Registry {
    }

    unsafe fn free_components<R>(
        _components: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where R: Registry {
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
    ) where R_: Registry {
        if identifier_iter.next().unwrap_unchecked() {
            component_map.insert(TypeId::of::<C>(), index);
            index += 1;
        }
        R::create_component_map_for_key(component_map, index, identifier_iter);
    }

    unsafe fn free_components<R_>(
        mut components: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) where R_: Registry {
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
