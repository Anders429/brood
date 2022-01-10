use crate::{component::Component, registry::Null};

pub trait RegistryLength {
    const LEN: usize;
}

impl RegistryLength for Null {
    const LEN: usize = 0;
}

impl<C, R> RegistryLength for (C, R)
where
    C: Component,
    R: RegistryLength,
{
    const LEN: usize = R::LEN + 1;
}
