use crate::{
    component::Component,
    internal::archetype,
    registry::{NullRegistry, Registry},
};
use alloc::vec::Vec;
use core::{
    any::type_name,
    fmt::{Debug, DebugList, DebugMap},
    mem::size_of,
};

pub trait RegistryDebug: Registry {
    unsafe fn extract_component_pointers<R>(
        index: usize,
        components: &[(*mut u8, usize)],
        pointers: &mut Vec<*const u8>,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where R: Registry;

    unsafe fn debug_components<'a, 'b, R>(
        pointers: &[*const u8],
        debug_map: &mut DebugMap<'a, 'b>,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where R: Registry;

    unsafe fn debug_identifier<'a, 'b, R>(
        debug_list: &mut DebugList<'a, 'b>,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where R: Registry;
}

impl RegistryDebug for NullRegistry {
    unsafe fn extract_component_pointers<R>(
        _index: usize,
        _components: &[(*mut u8, usize)],
        _pointers: &mut Vec<*const u8>,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where R: Registry {
    }

    unsafe fn debug_components<'a, 'b, R>(
        _pointers: &[*const u8],
        _debug_map: &mut DebugMap<'a, 'b>,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where R: Registry {
    }

    unsafe fn debug_identifier<'a, 'b, R>(
        _debug_list: &mut DebugList<'a, 'b>,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) where R: Registry {
    }
}

impl<C, R> RegistryDebug for (C, R)
where
    C: Component + Debug,
    R: RegistryDebug,
{
    unsafe fn extract_component_pointers<R_>(
        index: usize,
        mut components: &[(*mut u8, usize)],
        pointers: &mut Vec<*const u8>,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) where R_: Registry {
        if identifier_iter.next().unwrap_unchecked() {
            pointers.push(components.get_unchecked(0).0.add(index * size_of::<C>()));
            components = components.get_unchecked(1..);
        }

        R::extract_component_pointers(index, components, pointers, identifier_iter);
    }

    unsafe fn debug_components<'a, 'b, R_>(
        mut pointers: &[*const u8],
        debug_map: &mut DebugMap<'a, 'b>,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>
    ) where R_: Registry {
        if identifier_iter.next().unwrap_unchecked() {
            debug_map.entry(&type_name::<C>(), &*pointers.get_unchecked(0).cast::<C>());
            pointers = pointers.get_unchecked(1..);
        }

        R::debug_components(pointers, debug_map, identifier_iter);
    }

    unsafe fn debug_identifier<'a, 'b, R_>(
        debug_list: &mut DebugList<'a, 'b>,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) where R_: Registry {
        if identifier_iter.next().unwrap_unchecked() {
            debug_list.entry(&type_name::<C>());
        }

        R::debug_identifier(debug_list, identifier_iter);
    }
}
