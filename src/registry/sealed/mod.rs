//! Traits functions internal to the library.
//!
//! While [`Registry`] is a public trait, there are some implementation details that should be kept
//! private. This is done using a `Seal` trait, which is technically public, but within a private
//! module and therefore inaccessible to external users of the library.
//!
//! Additionally, making `Registry` a sealed trait blocks any external users from implementing it
//! on their own types that were never intended to be `Registry`s.
//!
//! Nothing within this module should be considered a part of the public API.
//!
//! [`Registry`]: crate::registry::Registry

mod assertions;
mod canonical;
#[cfg(feature = "rayon")]
mod claim;
mod length;
#[cfg(feature = "rayon")]
mod par_view;
mod storage;
mod view;

pub(crate) use canonical::Canonical;
#[cfg(feature = "rayon")]
pub(crate) use claim::Claims;
pub(crate) use length::Length;
#[cfg(feature = "rayon")]
pub(crate) use par_view::CanonicalParViews;
pub(crate) use view::CanonicalViews;

use crate::{
    component::Component,
    registry::Null,
};
use assertions::Assertions;
use storage::Storage;

/// A trait that is public but defined within a private module.
///
/// This effectively hides all function definitions, making them only usable internally within this
/// library. Additionally, no external types can implement this trait, and therefore no external
/// types can implement `Registry`.
///
/// While this trait specifically does not have any functions implemented, the traits it relies on
/// do. See the modules where they are defined for more details on the internal functionality
/// defined through these sealed traits.
#[cfg(feature = "rayon")]
pub trait Sealed: Assertions + Claims + Length + Storage {}
#[cfg(not(feature = "rayon"))]
pub trait Sealed: Assertions + Length + Storage {}

impl Sealed for Null {}

impl<C, R> Sealed for (C, R)
where
    C: Component,
    R: Sealed,
{
}
