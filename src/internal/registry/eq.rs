use crate::{
    component::Component,
    internal::archetype,
    registry::{NullRegistry, Registry},
};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

pub trait RegistryPartialEq: Registry {
    unsafe fn component_eq<R>(
        components_a: &[(*mut u8, usize)],
        components_b: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> bool where R: Registry;
}

impl RegistryPartialEq for NullRegistry {
    unsafe fn component_eq<R>(
        _components_a: &[(*mut u8, usize)],
        _components_b: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: impl archetype::IdentifierIterator<R>,
    ) -> bool where R: Registry {
        true
    }
}

impl<C, R> RegistryPartialEq for (C, R)
where
    C: Component + PartialEq,
    R: RegistryPartialEq,
{
    unsafe fn component_eq<R_>(
        mut components_a: &[(*mut u8, usize)],
        mut components_b: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: impl archetype::IdentifierIterator<R_>,
    ) -> bool where R_: Registry {
        if identifier_iter.next().unwrap_unchecked() {
            let component_column_a = components_a.get_unchecked(0);
            let component_column_b = components_b.get_unchecked(0);

            if ManuallyDrop::new(Vec::from_raw_parts(
                component_column_a.0,
                length,
                component_column_a.1,
            )) != ManuallyDrop::new(Vec::from_raw_parts(
                component_column_b.0,
                length,
                component_column_b.1,
            )) {
                return false;
            }

            components_a = components_a.get_unchecked(1..);
            components_b = components_b.get_unchecked(1..);
        }

        R::component_eq(
            components_a,
            components_b,
            length,
            identifier_iter,
        )
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
