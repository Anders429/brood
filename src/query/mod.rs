//! Queries over [`World`]s.
//!
//! Entities within a `World` are difficult to interact with directly due to being made of
//! heterogeneous lists of [`Component`]s. Therefore, queries can be executed to give [`Views`] of
//! `Component`s within the entities stored in a `World`.
//!
//! Queries are made up of `Views`, giving access to `Component`s, and [`Filter`]s which can filter
//! which entities are viewed. Query results are returned as heterogeneous lists, so the
//! [`result!`] macro is provided to unpack the results.
//!
//! # Example
//! The below example queries mutably for the component `Foo`, immutably for the component `Bar`,
//! and filters out entities that do not have the component `Baz`.
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
//! // Define components.
//! struct Foo(u32);
//! struct Bar(bool);
//! struct Baz(f64);
//!
//! type Registry = Registry!(Foo, Bar, Baz);
//!
//! let mut world = World::<Registry>::new();
//! world.insert(entity!(Foo(42), Bar(true), Baz(1.5)));
//!
//! for result!(foo, bar) in world
//!     .query(Query::<Views!(&mut Foo, &Bar), filter::Has<Baz>>::new())
//!     .iter
//! {
//!     // Do something.
//! }
//! ```
//!
//! [`Component`]: crate::component::Component
//! [`Filter`]: crate::query::filter::Filter
//! [`result!`]: crate::query::result!
//! [`Views`]: trait@crate::query::view::Views
//! [`World`]: crate::world::World

pub mod entries;
pub mod filter;
pub mod result;
pub mod view;

#[doc(inline)]
pub use entries::Entries;
#[doc(inline)]
pub use result::{
    result,
    Result,
};
#[doc(inline)]
pub use view::inner::Views;

use core::{
    fmt,
    marker::PhantomData,
};

/// Defines a query to be run over a world.
///
/// This defines either a regular or parallel query (parallel requires the `rayon` feature to be
/// enabled). It is essentially a marker type, simply providing the types to the calls to
/// [`query()`] to make the API as simple to use as possible.
///
/// # Example
/// ``` rust
/// use brood::{
///     entity,
///     query::{
///         filter,
///         result,
///         Views,
///     },
///     Query,
///     Registry,
///     World,
/// };
///
/// // Define components.
/// struct Foo(u32);
/// struct Bar(bool);
/// struct Baz(f64);
///
/// type Registry = Registry!(Foo, Bar, Baz);
///
/// let mut world = World::<Registry>::new();
/// world.insert(entity!(Foo(42), Bar(true), Baz(1.5)));
///
/// for result!(foo, bar) in world
///     .query(Query::<Views!(&mut Foo, &Bar), filter::Has<Baz>>::new())
///     .iter
/// {
///     // Do something.
/// }
/// ```
///
/// [`query()`]: crate::world::World::query()
pub struct Query<Views, Filters = filter::None, ResourceViews = view::Null, EntryViews = view::Null>
{
    view: PhantomData<Views>,
    filter: PhantomData<Filters>,
    resource_views: PhantomData<ResourceViews>,
    entry_views: PhantomData<EntryViews>,
}

impl<Views, Filters, ResourceViews, EntryViews> Query<Views, Filters, ResourceViews, EntryViews> {
    /// Creates a new `Query`.
    ///
    /// When creating a query, you must specify the views type `V`, and can optionally specify the
    /// filter type `F`. If no filter is specified, the default filter, [`filter::None`], will be
    /// used.
    #[must_use]
    pub fn new() -> Self {
        Self {
            view: PhantomData,
            filter: PhantomData,
            resource_views: PhantomData,
            entry_views: PhantomData,
        }
    }
}

impl<Views, Filters, ResourceViews, EntryViews> Default
    for Query<Views, Filters, ResourceViews, EntryViews>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Views, Filters, ResourceViews, EntryViews> Clone
    for Query<Views, Filters, ResourceViews, EntryViews>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Views, Filters, ResourceViews, EntryViews> PartialEq
    for Query<Views, Filters, ResourceViews, EntryViews>
{
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<Views, Filters, ResourceViews, EntryViews> Eq
    for Query<Views, Filters, ResourceViews, EntryViews>
{
}

impl<Views, Filters, ResourceViews, EntryViews> Copy
    for Query<Views, Filters, ResourceViews, EntryViews>
{
}

impl<Views, Filters, ResourceViews, EntryViews> fmt::Debug
    for Query<Views, Filters, ResourceViews, EntryViews>
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Query").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Query;
    use crate::query::Views;
    use alloc::format;

    #[test]
    fn query_default() {
        assert_eq!(Query::<Views!()>::default(), Query::<Views!()>::new());
    }

    #[test]
    fn query_clone() {
        let query = Query::<Views!()>::new();

        assert_eq!(query.clone(), query);
    }

    #[test]
    fn query_debug() {
        assert_eq!(format!("{:?}", Query::<Views!()>::new()), "Query");
    }
}
