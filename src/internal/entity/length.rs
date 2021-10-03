use crate::{component::Component, entity::NullEntity};

pub trait EntityLength {
    const LEN: usize;
}

impl EntityLength for NullEntity {
    const LEN: usize = 0;
}

impl<C, E> EntityLength for (C, E)
where
    C: Component,
    E: EntityLength,
{
    const LEN: usize = E::LEN + 1;
}
