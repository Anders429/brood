use crate::{
    component::Component,
    registry::{
        Null,
        Registry,
    },
};

pub trait RegistrySync: Registry {}

impl RegistrySync for Null {}

impl<C, R> RegistrySync for (C, R)
where
    C: Component + Sync,
    R: RegistrySync,
{
}
