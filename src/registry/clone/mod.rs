mod sealed;

use crate::{
    component,
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

impl<Component, Registry> Clone for (Component, Registry)
where
    Component: clone::Clone + component::Component,
    Registry: Clone,
{
}
