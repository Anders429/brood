use crate::{
    component::Component,
    registry::{NullRegistry, Registry},
};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

pub trait RegistryPartialEq: Registry {
    unsafe fn component_eq(
        components_a: &[(*mut u8, usize)],
        components_b: &[(*mut u8, usize)],
        length: usize,
        identifier_iter: impl Iterator<Item = bool>,
    ) -> bool;
}

impl RegistryPartialEq for NullRegistry {
    unsafe fn component_eq(
        _components_a: &[(*mut u8, usize)],
        _components_b: &[(*mut u8, usize)],
        _length: usize,
        _identifier_iter: impl Iterator<Item = bool>,
    ) -> bool {
        true
    }
}

impl<C, R> RegistryPartialEq for (C, R)
where
    C: Component + PartialEq,
    R: RegistryPartialEq,
{
    unsafe fn component_eq(
        mut components_a: &[(*mut u8, usize)],
        mut components_b: &[(*mut u8, usize)],
        length: usize,
        mut identifier_iter: impl Iterator<Item = bool>,
    ) -> bool {
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
