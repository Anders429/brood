mod byte_length;
mod debug;
mod eq;
mod length;
#[cfg(feature = "serde")]
mod serde;
mod storage;

#[cfg(feature = "serde")]
pub(crate) use self::serde::{EntityDeserialize, EntitySerialize};
pub(crate) use debug::EntityDebug;
pub(crate) use eq::{EntityEq, EntityPartialEq};

use crate::{component::Component, entity::NullEntity};
use byte_length::EntityByteLength;
use length::EntityLength;
use storage::EntityStorage;

pub trait EntitySeal: EntityByteLength + EntityLength + EntityStorage {}

impl EntitySeal for NullEntity {}

impl<C, E> EntitySeal for (C, E)
where
    C: Component,
    E: EntitySeal,
{
}
