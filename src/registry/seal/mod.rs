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
mod filter;
mod length;
mod storage;

pub(crate) use canonical::Canonical;
pub(crate) use filter::Filter;

use crate::{component::Component, registry::Null};
use assertions::Assertions;
use length::Length;
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
pub trait Seal: Assertions + Length + Storage {}

impl Seal for Null {}

impl<C, R> Seal for (C, R)
where
    C: Component,
    R: Seal,
{
}
