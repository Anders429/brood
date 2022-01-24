use crate::{component::Component, registry::Null};

pub trait Length {
    const LEN: usize;
}

impl Length for Null {
    const LEN: usize = 0;
}

impl<C, R> Length for (C, R)
where
    C: Component,
    R: Length,
{
    const LEN: usize = R::LEN + 1;
}
