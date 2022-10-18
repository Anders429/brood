//! Traits indicating that a [`Registry`] contains a specific type.
//!
//! Being able to prove that a registry contains a specific component means that the component's
//! presence can be verified at compile-time. Additionally, being able to prove that a registry
//! contains a heterogeneous list of components allows transforming that list into a canonical form
//! at compile-time, paving the way for storage optimizations.

pub(crate) mod entities;
pub(crate) mod entity;
pub(crate) mod filter;
#[cfg(feature = "rayon")]
pub(crate) mod par_views;
pub(crate) mod views;

mod component;

pub use component::ContainsComponent;
pub use entities::ContainsEntities;
pub use entity::ContainsEntity;
pub use filter::ContainsFilter;
#[cfg(feature = "rayon")]
pub use par_views::ContainsParViews;
pub use views::ContainsViews;

/// Type marker for a component contained in an entity.
pub enum Contained {}

/// Type marker for a component not contained in an entity.
pub enum NotContained {}

/// Defines the end of a heterogeneous list of containments.
pub enum Null {}

pub enum EntityIdentifierMarker {}
