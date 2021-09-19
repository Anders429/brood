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
    unsafe fn debug<'a, 'b, E, R>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype: &Box<dyn Any>,
        debug_map: &mut DebugMap<'a, 'b>,
        entity: PhantomData<E>,
        registry: PhantomData<R>,
    ) where
        R: Registry,
        E: EntityDebug;
}

impl RegistryDebug for NullRegistry {
    unsafe fn debug<'a, 'b, E, R>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype: &Box<dyn Any>,
        debug_map: &mut DebugMap<'a, 'b>,
        entity: PhantomData<E>,
        registry: PhantomData<R>,
    ) where
        R: Registry,
        E: EntityDebug,
    {
        debug_map.entry(
            &type_name::<E>(),
            &archetype.downcast_ref_unchecked::<Archetype<E>>(),
        );
    }
}

impl<C, R1> RegistryDebug for (C, R1)
where
    C: Component + Debug,
    R1: RegistryDebug,
{
    unsafe fn debug<'a, 'b, E, R2>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype: &Box<dyn Any>,
        debug_map: &mut DebugMap<'a, 'b>,
        entity: PhantomData<E>,
        registry: PhantomData<R2>,
    ) where
        R2: Registry,
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
            R1::debug::<(C, E), R2>(
                key,
                new_index,
                new_bit,
                archetype,
                debug_map,
                PhantomData,
                PhantomData,
            );
        } else {
            R1::debug::<E, R2>(
                key,
                new_index,
                new_bit,
                archetype,
                debug_map,
                PhantomData,
                PhantomData,
            );
        }
    }
}
