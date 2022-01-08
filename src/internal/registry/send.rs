use crate::{
    component::Component,
    registry::{NullRegistry, Registry},
};

pub trait RegistrySend: Registry {}

impl RegistrySend for NullRegistry {}

impl<C, R> RegistrySend for (C, R)
where
    C: Component + Send,
    R: RegistrySend,
{
}
