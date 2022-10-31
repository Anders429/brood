mod sealed;

use crate::{
    component::Component,
    registry::Null,
};
use sealed::Sealed;

/// A registry whose components implement [`Deserialize`].
///
/// This is a supertrait to the `Deserialize` trait. It is always implemented when all components
/// implement `Deserialize`.
///
/// [`Deserialize`]: serde::Deserialize
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
pub trait Deserialize<'de>: Sealed<'de> {}

impl<'de> Deserialize<'de> for Null {}

impl<'de, C, R> Deserialize<'de> for (C, R)
where
    C: Component + serde::Deserialize<'de>,
    R: Deserialize<'de>,
{
}
