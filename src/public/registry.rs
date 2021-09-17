use crate::{component::Component, internal::registry::RegistrySeal};

pub struct NullRegistry;

pub trait Registry: RegistrySeal {}

impl Registry for NullRegistry {}

impl<C, R> Registry for (C, R)
where
    C: Component,
    R: Registry,
{
}

#[macro_export]
macro_rules! registry {
    ($component:ty $(,$components:ty)* $(,)?) => {
        ($component, registry!($($components,)*))
    };
    () => {
        $crate::registry::NullRegistry
    };
}
