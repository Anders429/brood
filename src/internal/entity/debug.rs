use crate::{
    component::Component,
    entity::{Entity, NullEntity},
};
use alloc::vec::Vec;
use core::{
    any::type_name,
    fmt::{Debug, DebugMap},
    mem::ManuallyDrop,
};

pub trait EntityDebug: Entity {
    unsafe fn debug<'a, 'b, 'c>(
        components: &[(*mut u8, usize)],
        length: usize,
        debug_map: &'c mut DebugMap<'a, 'b>,
    ) -> &'c mut DebugMap<'a, 'b>;
}

impl EntityDebug for NullEntity {
    unsafe fn debug<'a, 'b, 'c>(
        components: &[(*mut u8, usize)],
        length: usize,
        debug_map: &'c mut DebugMap<'a, 'b>,
    ) -> &'c mut DebugMap<'a, 'b> {
        debug_map
    }
}

impl<C, E> EntityDebug for (C, E)
where
    C: Component + Debug,
    E: EntityDebug,
{
    unsafe fn debug<'a, 'b, 'c>(
        components: &[(*mut u8, usize)],
        length: usize,
        debug_map: &'c mut DebugMap<'a, 'b>,
    ) -> &'c mut DebugMap<'a, 'b> {
        let component_column = components.get_unchecked(0);
        let v = ManuallyDrop::new(Vec::<C>::from_raw_parts(
            component_column.0.cast::<C>(),
            length,
            component_column.1,
        ));
        E::debug(
            components.get_unchecked(1..),
            length,
            debug_map.entry(&type_name::<C>(), &*v),
        )
    }
}
