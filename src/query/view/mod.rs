//! Views over entities.
//!
//! Along with [`Filter`]s, [`Views`] are what make up a query over entities stored wihin a
//! [`World`]. `Views` are how queries specify what [`Component`]s should be borrowed within query
//! results.
//!
//! There are five types of [`View`]s that can be used when defining a query:
//! - **`&C`** - Borrows the `Component` `C` immutably, filtering out any entities that do not
//! contain `C`.
//! - **`&mut C`** - Borrows the `Component` `C` mutably, filtering out any entities that do not
//! contain `C`.
//! - **`Option<&C>`** - Borrows the `Component` `C` immutably if present in the entity. Returns
//! [`None`] otherwise.
//! - **`Option<&mut C>`** - Borrows the `Component` `C` mutably if present in the entity. Returns
//! [`None`] otherwise.
//! - **[`entity::Identifier`]** - Returns the `entity::Identifier` of each entity in the query
//! results.
//!
//! `Views` is a heterogeneous list of individual `View`s. Therefore, it is easiest to define them
//! using the [`Views!`] macro.
//!
//! # Example
//! ``` rust
//! use brood::{
//!     entity,
//!     query::Views,
//! };
//!
//! // Define components.
//! struct Foo(u32);
//! struct Bar(bool);
//! struct Baz(f64);
//!
//! type Views<'a> = Views!(&'a mut Foo, &'a Bar, Option<&'a Baz>, entity::Identifier);
//! ```
//!
//! Note that the lifetime `'a` can often be omitted when [`query`]ing a [`World`], but is required
//! when defining a [`System`].
//!
//! [`Component`]: crate::component::Component
//! [`entity::Identifier`]: crate::entity::Identifier
//! [`Filter`]: crate::query::filter::Filter
//! [`query`]: crate::world::World::query()
//! [`System`]: crate::system::System
//! [`View`]: crate::query::view::View
//! [`Views`]: trait@crate::query::view::Views
//! [`Views!`]: crate::query::Views!
//! [`World`]: crate::world::World.

#[cfg(feature = "rayon")]
pub(crate) mod claim;
pub(crate) mod resource;

mod contains;
mod disjoint;
#[cfg(feature = "rayon")]
mod merge;
#[cfg(feature = "rayon")]
mod par;
mod sealed;
mod subset;

pub use contains::ContainsFilter;
pub use disjoint::Disjoint;
#[cfg(feature = "rayon")]
pub use par::{
    ParView,
    ParViews,
};
pub use subset::SubSet;

#[cfg(feature = "rayon")]
pub(crate) use claim::{
    Claim,
    Claims,
};
#[cfg(feature = "rayon")]
pub(crate) use merge::Merge;
#[cfg(feature = "rayon")]
pub(crate) use par::{
    ParViewsSeal,
    RepeatNone,
};
pub(crate) use sealed::ViewsSealed;

use crate::{
    component,
    entity,
    hlist::define_null,
};
use sealed::ViewSealed;

/// A view over a single aspect of an entity.
///
/// Here, the world "aspect" means either a [`Component`] or the entity's [`Identifier`].
/// Specifically, `View` is implemented for each of the following five types, providing the
/// specified view into the entity:
/// - **`&C`** - Borrows the `Component` `C` immutably, filtering out any entities that do not
/// contain `C`.
/// - **`&mut C`** - Borrows the `Component` `C` mutably, filtering out any entities that do not
/// contain `C`.
/// - **`Option<&C>`** - Borrows the `Component` `C` immutably if present in the entity. Returns
/// [`None`] otherwise.
/// - **`Option<&mut C>`** - Borrows the `Component` `C` mutably if present in the entity. Returns
/// [`None`] otherwise.
/// - **[`entity::Identifier`]** - Returns the `entity::Identifier` of each entity in the query
/// results.
///
/// # Example
/// ``` rust
/// // Define a component.
/// struct Foo(usize);
///
/// // Define a view over that component.
/// type FooView<'a> = &'a Foo;
/// ```
///
/// Note that a single `View` by itself isn't very useful. To be usable in querying a [`World`],
/// a [`Views`] heterogeneous list must be used. It is recommended to use the [`Views!`] macro to
/// construct this heterogeneous list.
///
/// ``` rust
/// use brood::query::Views;
///
/// // Define components.
/// struct Foo(u32);
/// struct Bar(bool);
///
/// type Views<'a> = Views!(&'a mut Foo, &'a Bar);
/// ```
///
/// [`Component`]: crate::component::Component
/// [`Identifier`]: crate::entity::Identifier
/// [`Views`]: trait@crate::query::view::Views
/// [`Views!`]: crate::query::Views!
/// [`World`]: crate::world::World
pub trait View<'a>: ViewSealed<'a> {}

impl<'a, Component> View<'a> for &'a Component where Component: component::Component {}

impl<'a, Component> View<'a> for &'a mut Component where Component: component::Component {}

impl<'a, Component> View<'a> for Option<&'a Component> where Component: component::Component {}

impl<'a, Component> View<'a> for Option<&'a mut Component> where Component: component::Component {}

impl<'a> View<'a> for entity::Identifier {}

define_null!();

/// A heterogeneous list of [`View`]s.
///
/// Along with [`Filter`]s, `Views` are what make up a query over entities stored wihin a
/// [`World`]. `Views` are how queries specify what [`Component`]s should be borrowed within query
/// results.
///
/// Note that while multiple immutable borrows of `Component`s are allowed within a `Views`, if a
/// component is borrowed mutably in a `Views` it cannot be borrowed again within the same `Views`.
/// In other words, borrows in `Views` must follow Rust's
/// [borrowing rules](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html).
///
/// As `Views` is a heterogeneous list, it is most easily constructed using the [`Views!`] macro.
///
/// # Example
/// ``` rust
/// use brood::query::Views;
///
/// // Define components.
/// struct Foo(u32);
/// struct Bar(bool);
///
/// type Views<'a> = Views!(&'a mut Foo, &'a Bar);
/// ```
///
/// [`Component`]: crate::component::Component
/// [`Filter`]: crate::query::filter::Filter
/// [`View`]: crate::query::view::View
/// [`Views!`]: crate::query::Views!
/// [`World`]: crate::world::World
pub trait Views<'a>: ViewsSealed<'a> {}

impl<'a> Views<'a> for Null {}

impl<'a, View, Views> self::Views<'a> for (View, Views)
where
    View: self::View<'a>,
    Views: self::Views<'a>,
{
}

pub(crate) mod inner {
    use crate::doc;
    doc::non_root_macro! {
        /// Creates a set of [`View`]s over components.
        ///
        /// These views can be used to [`query`] the components stored within a [`World`]. They can also be
        /// used when defining [`System`]s to be run over components stored in a [`World`].
        ///
        /// See the documentation for [`View`] to learn more about what kinds of `View`s can be created.
        ///
        /// # Example
        /// ``` rust
        /// use brood::query::Views;
        ///
        /// // Define components.
        /// struct Foo(u32);
        /// struct Bar(bool);
        ///
        /// type Views<'a> = Views!(&'a mut Foo, &'a Bar);
        /// ```
        ///
        /// Note that the lifetime `'a` can often be omitted when [`query`]ing a [`World`], but is required
        /// when defining a [`System`].
        ///
        /// [`query`]: crate::world::World::query()
        /// [`System`]: crate::system::System
        /// [`View`]: crate::query::view::View
        /// [`World`]: crate::world::World
        macro_rules! Views {
            ($view:ty $(,$views:ty)* $(,)?) => (
                ($view, $crate::query::view::Views!($($views,)*))
            );
            () => (
                $crate::query::view::Null
            );
        }
    }
}

pub use inner::Views;
