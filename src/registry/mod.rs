mod debug;
mod eq;
mod seal;
mod send;
#[cfg(feature = "serde")]
mod serde;
mod sync;

#[cfg(feature = "serde")]
pub(crate) use self::serde::{RegistryDeserialize, RegistrySerialize};
pub(crate) use debug::RegistryDebug;
pub(crate) use eq::{RegistryEq, RegistryPartialEq};
pub(crate) use send::RegistrySend;
pub(crate) use sync::RegistrySync;

use crate::{component::Component, hlist::define_null};
use seal::Seal;

define_null!();

pub trait Registry: Seal {}

impl Registry for Null {}

impl<C, R> Registry for (C, R)
where
    C: Component,
    R: Registry,
{
}

#[macro_export]
macro_rules! registry {
    ($component:ty $(,$components:ty)* $(,)?) => {
        ($component, $crate::registry!($($components,)*))
    };
    () => {
        $crate::registry::Null
    };
}
