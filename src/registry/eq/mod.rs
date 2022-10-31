mod sealed;

use crate::{
    component::Component,
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

impl<C, R> PartialEq for (C, R)
where
    C: Component + cmp::PartialEq,
    R: PartialEq,
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

impl<C, R> Eq for (C, R)
where
    C: Component + cmp::Eq,
    R: Eq,
{
}
