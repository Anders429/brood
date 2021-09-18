use crate::{
    component::Component,
    entity::{Entity, NullEntity},
};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

pub trait EntityPartialEq: Entity {
    unsafe fn eq(
        components_a: &[(*mut u8, usize)],
        components_b: &[(*mut u8, usize)],
        length: usize,
    ) -> bool;
}

impl EntityPartialEq for NullEntity {
    unsafe fn eq(
        components_a: &[(*mut u8, usize)],
        components_b: &[(*mut u8, usize)],
        length: usize,
    ) -> bool {
        true
    }
}

impl<C, E> EntityPartialEq for (C, E)
where
    C: Component + PartialEq,
    E: EntityPartialEq,
{
    unsafe fn eq(
        components_a: &[(*mut u8, usize)],
        components_b: &[(*mut u8, usize)],
        length: usize,
    ) -> bool {
        let components_column_a = components_a.get_unchecked(0);
        let components_column_b = components_b.get_unchecked(0);

        let v_a = ManuallyDrop::new(Vec::<C>::from_raw_parts(
            components_column_a.0.cast::<C>(),
            length,
            components_column_a.1,
        ));
        let v_b = ManuallyDrop::new(Vec::<C>::from_raw_parts(
            components_column_b.0.cast::<C>(),
            length,
            components_column_b.1,
        ));

        if v_a == v_b {
            E::eq(
                components_a.get_unchecked(1..),
                components_b.get_unchecked(1..),
                length,
            )
        } else {
            false
        }
    }
}

pub trait EntityEq: EntityPartialEq {}

impl EntityEq for NullEntity {}

impl<C, E> EntityEq for (C, E)
where
    C: Component + Eq,
    E: EntityEq,
{
}

#[cfg(test)]
mod tests {
    // TODO.
}
