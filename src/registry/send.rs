use crate::{
    component::Component,
    registry::{
        Null,
        Registry,
    },
};

pub trait RegistrySend: Registry {}

impl RegistrySend for Null {}

impl<C, R> RegistrySend for (C, R)
where
    C: Component + Send,
    R: RegistrySend,
{
}
