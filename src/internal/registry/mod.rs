mod debug;
mod eq;
mod length;
#[cfg(feature = "serde")]
mod serde;
mod storage;
mod view;

#[cfg(feature = "serde")]
pub(crate) use self::serde::{RegistryDeserialize, RegistrySerialize};
pub(crate) use debug::RegistryDebug;
pub(crate) use eq::{RegistryEq, RegistryPartialEq};

use crate::{component::Component, registry::NullRegistry};
use length::RegistryLength;
use storage::RegistryStorage;
use view::RegistryView;

pub trait RegistrySeal: RegistryLength + RegistryStorage + RegistryView {}

impl RegistrySeal for NullRegistry {}

impl<C, R> RegistrySeal for (C, R)
where
    C: Component,
    R: RegistrySeal,
{
}
