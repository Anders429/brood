mod sealed;

use crate::{
    component,
    registry::Null,
};
use sealed::Sealed;

/// A registry whose components implement [`Serialize`].
///
/// This is a supertrait to the `Serialize` trait. It is always implemented when all components
/// implement `Serialize`.
///
/// [`Serialize`]: serde::Serialize
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
pub trait Serialize: Sealed {}

impl Serialize for Null {}

impl<Component, Registry> Serialize for (Component, Registry)
where
    Component: component::Component + serde::Serialize,
    Registry: Serialize,
{
}
