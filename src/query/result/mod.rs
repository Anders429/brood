//! Results of queries.
//!
//! The primary way to interact with entities stored within a [`World`] is to query their
//! [`Component`]s using [`Views`] and [`Filter`]s. This module handles interaction with the
//! results of those queries.
//!
//! As `Component`s should be allowed to be queried arbitrarily, with any amount of `Component`s
//! and any amount of `Filter`s being requested at once, the returned results of queries must be
//! just as flexible. Therefore, the returned results are in the form of heterogeneous lists. In
//! order to unpack these values into usable identifiers, a [`result!`] macro is provided to remove
//! the unpleasant boilerplate.
//!
//! # Example
//! The following example queries a `World` for all entities containing two `Component`s, giving
//! `View`s over both `Component`s. One of these `View`s is mutable, allowing the component to be
//! modified during iteration.
//!
//! ``` rust
//! use brood::{
//!     entity,
//!     query::{
//!         filter,
//!         result,
//!         Views,
//!     },
//!     Query,
//!     Registry,
//!     World,
//! };
//!
//! struct Foo(u32);
//! struct Bar(bool);
//!
//! type Registry = Registry!(Foo, Bar);
//!
//! let mut world = World::<Registry>::new();
//! world.insert(entity!(Foo(42), Bar(true)));
//!
//! for result!(foo, bar) in world.query(Query::<Views!(&mut Foo, &Bar)>::new()) {
//!     if bar.0 {
//!         foo.0 += 1;
//!     }
//! }
//! ```
//!
//! [`Component`]: crate::component::Component
//! [`Filter`]: crate::query::filter::Filter
//! [`result!`]: crate::query::result!
//! [`Views`]: crate::query::view::Views
//! [`World`]: crate::world::World

pub(crate) mod get;
pub(crate) mod reshape;

mod iter;
#[cfg(feature = "rayon")]
mod par_iter;
mod sealed;

pub use iter::Iter;
#[cfg(feature = "rayon")]
pub use par_iter::ParIter;

pub(crate) use get::Get;
pub(crate) use reshape::Reshape;
#[cfg(feature = "rayon")]
pub(crate) use sealed::ParResults;
pub(crate) use sealed::Results;

use crate::doc;

doc::non_root_macro! {
    /// Defines identifiers to match items returned by a [`result::Iter`] iterator.
    ///
    /// This allows matching identifiers with the heterogeneous lists iterated by the `result::Iter`
    /// iterator.
    ///
    /// # Example
    /// ``` rust
    /// use brood::{entity, query::{filter, result, Views}, Registry, Query, World};
    ///
    /// struct Foo(u32);
    /// struct Bar(bool);
    ///
    /// type Registry = Registry!(Foo, Bar);
    ///
    /// let mut world = World::<Registry>::new();
    /// world.insert(entity!(Foo(42), Bar(true)));
    ///
    /// for result!(foo, bar) in world.query(Query::<Views!(&mut Foo, &Bar)>::new()) {
    ///     // ...
    /// }
    /// ```
    ///
    /// [`result::Iter`]: crate::query::result::Iter
    macro_rules! result {
        () => (
            _
        );
        ($component:ident $(,$components:ident)* $(,)?) => (
            ($component, $crate::query::result!($($components,)*))
        );
    }
}
