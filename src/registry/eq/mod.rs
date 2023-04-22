mod sealed;

use crate::{
    component,
    registry::Null,
};
use core::cmp;
use sealed::Sealed;

/// A registry whose components implement [`PartialEq`].
///
/// This is a supertrait to the `PartialEq` trait. It is always implemented when all components
/// implement `PartialEq`.
///
/// [`PartialEq`]: core::cmp::PartialEq
pub trait PartialEq: Sealed {}

impl PartialEq for Null {}

impl<Component, Registry> PartialEq for (Component, Registry)
where
    Component: component::Component + cmp::PartialEq,
    Registry: PartialEq,
{
}

/// A registry whose components implement [`Eq`].
///
/// This is a supertrait to the `Eq` trait. It is always implemented when all components
/// implement `Eq`.
///
/// [`Eq`]: core::cmp::Eq
pub trait Eq: PartialEq {}

impl Eq for Null {}

impl<Component, Registry> Eq for (Component, Registry)
where
    Component: component::Component + cmp::Eq,
    Registry: Eq,
{
}
