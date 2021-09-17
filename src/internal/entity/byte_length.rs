use crate::{component::Component, entity::NullEntity};
use core::mem::size_of;

pub trait EntityByteLength {
    const BYTE_LEN: usize;
}

impl EntityByteLength for NullEntity {
    const BYTE_LEN: usize = 0;
}

impl<C, E> EntityByteLength for (C, E)
where
    C: Component,
    E: EntityByteLength,
{
    const BYTE_LEN: usize = size_of::<C>() + E::BYTE_LEN;
}
