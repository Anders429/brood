mod debug;
mod length;
mod storage;

pub(crate) use debug::RegistryDebug;

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
