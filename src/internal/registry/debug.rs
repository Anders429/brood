use crate::{
    component::Component,
    internal::{archetype::Archetype, entity::EntityDebug},
    registry::{NullRegistry, Registry},
};
use alloc::boxed::Box;
use core::{
    any::{type_name, Any},
    fmt::{Debug, DebugMap},
    marker::PhantomData,
};
use unsafe_any::UnsafeAnyExt;

pub trait RegistryDebug: Registry {
    unsafe fn debug<'a, 'b, E>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype: &Box<dyn Any>,
        debug_map: &mut DebugMap<'a, 'b>,
        entity: PhantomData<E>,
    ) where
        E: EntityDebug;
}

impl RegistryDebug for NullRegistry {
    unsafe fn debug<'a, 'b, E>(
        _key: &[u8],
        _index: usize,
        _bit: usize,
        archetype: &Box<dyn Any>,
        debug_map: &mut DebugMap<'a, 'b>,
        _entity: PhantomData<E>,
    ) where
        E: EntityDebug,
    {
        debug_map.entry(
            &type_name::<E>(),
            &archetype.downcast_ref_unchecked::<Archetype<E>>(),
        );
    }
}

impl<C, R> RegistryDebug for (C, R)
where
    C: Component + Debug,
    R: RegistryDebug,
{
    unsafe fn debug<'a, 'b, E>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype: &Box<dyn Any>,
        debug_map: &mut DebugMap<'a, 'b>,
        _entity: PhantomData<E>,
    ) where
        E: EntityDebug,
    {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R::debug::<(C, E)>(
                key,
                new_index,
                new_bit,
                archetype,
                debug_map,
                PhantomData,
            );
        } else {
            R::debug::<E>(
                key,
                new_index,
                new_bit,
                archetype,
                debug_map,
                PhantomData,
            );
        }
    }
}
