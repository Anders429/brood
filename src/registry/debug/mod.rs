mod sealed;

use crate::{
    component::Component,
    registry::Null,
};
use core::fmt;
use sealed::Sealed;

/// A registry whose components implement [`Debug`].
///
/// This is a supertrait to the `Debug` trait. It is always implemented when all components
/// implement `Debug`.
///
/// [`Debug`]: core::fmt::Debug
pub trait Debug: Sealed {}

impl Debug for Null {}

impl<C, R> Debug for (C, R)
where
    C: Component + fmt::Debug,
    R: Debug,
{
}
