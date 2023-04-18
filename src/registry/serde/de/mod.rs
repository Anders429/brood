mod sealed;

use crate::{
    component,
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

impl<'de, Component, Registry> Deserialize<'de> for (Component, Registry)
where
    Component: component::Component + serde::Deserialize<'de>,
    Registry: Deserialize<'de>,
{
}
