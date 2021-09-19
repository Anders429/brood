use crate::{
    component::Component,
    internal::{archetype::Archetype, entity::EntityPartialEq},
    registry::{NullRegistry, Registry},
};
use alloc::boxed::Box;
use core::{any::Any, marker::PhantomData};
use unsafe_any::UnsafeAnyExt;

pub trait RegistryPartialEq: Registry {
    unsafe fn eq<E, R>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype_a: &Box<dyn Any>,
        archetype_b: &Box<dyn Any>,
        entity: PhantomData<E>,
        registry: PhantomData<R>,
    ) -> bool
    where
        E: EntityPartialEq,
        R: Registry;
}

impl RegistryPartialEq for NullRegistry {
    unsafe fn eq<E, R>(
        _key: &[u8],
        _index: usize,
        _bit: usize,
        archetype_a: &Box<dyn Any>,
        archetype_b: &Box<dyn Any>,
        _entity: PhantomData<E>,
        _registry: PhantomData<R>,
    ) -> bool
    where
        E: EntityPartialEq,
        R: Registry,
    {
        archetype_a.downcast_ref_unchecked::<Archetype<E>>()
            == archetype_b.downcast_ref_unchecked::<Archetype<E>>()
    }
}

impl<C, R1> RegistryPartialEq for (C, R1)
where
    C: Component + PartialEq,
    R1: RegistryPartialEq,
{
    unsafe fn eq<E, R2>(
        key: &[u8],
        index: usize,
        bit: usize,
        archetype_a: &Box<dyn Any>,
        archetype_b: &Box<dyn Any>,
        _entity: PhantomData<E>,
        _registry: PhantomData<R2>,
    ) -> bool
    where
        E: EntityPartialEq,
        R2: Registry,
    {
        let mut new_bit = bit + 1;
        let new_index = if bit >= 8 {
            new_bit %= 8;
            index + 1
        } else {
            index
        };

        if key.get_unchecked(index) & (1 << bit) != 0 {
            R1::eq::<(C, E), R2>(
                key,
                new_index,
                new_bit,
                archetype_a,
                archetype_b,
                PhantomData,
                PhantomData,
            )
        } else {
            R1::eq::<E, R2>(
                key,
                new_index,
                new_bit,
                archetype_a,
                archetype_b,
                PhantomData,
                PhantomData,
            )
        }
    }
}

pub trait RegistryEq: RegistryPartialEq {}

impl RegistryEq for NullRegistry {}

impl<C, R> RegistryEq for (C, R)
where
    C: Component + Eq,
    R: RegistryEq,
{
}
