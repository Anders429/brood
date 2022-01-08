mod debug;
mod eq;
mod length;
mod send;
#[cfg(feature = "serde")]
mod serde;
mod storage;
mod sync;

#[cfg(feature = "serde")]
pub(crate) use self::serde::{RegistryDeserialize, RegistrySerialize};
pub(crate) use debug::RegistryDebug;
pub(crate) use eq::{RegistryEq, RegistryPartialEq};
pub(crate) use send::RegistrySend;
pub(crate) use sync::RegistrySync;

use crate::{component::Component, registry::NullRegistry};
use length::RegistryLength;
use storage::RegistryStorage;

pub trait RegistrySeal: RegistryLength + RegistryStorage {}

impl RegistrySeal for NullRegistry {}

impl<C, R> RegistrySeal for (C, R)
where
    C: Component,
    R: RegistrySeal,
{
}
