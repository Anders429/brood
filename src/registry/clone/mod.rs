mod sealed;

use crate::{
    component::Component,
    registry::Null,
};
use core::clone;
use sealed::Sealed;

/// A registry whose components implement [`Clone`].
///
/// This is a supertrait to the `Clone` trait. It is always implemented when all components
/// implement `Clone`.
pub trait Clone: Sealed {}

impl Clone for Null {}

impl<C, R> Clone for (C, R)
where
    C: clone::Clone + Component,
    R: Clone,
{
}
