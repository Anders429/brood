mod sealed;

use crate::{
    component::Component,
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

impl<C, R> Serialize for (C, R)
where
    C: Component + serde::Serialize,
    R: Serialize,
{
}
