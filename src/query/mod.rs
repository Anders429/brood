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
//!     query::{filter, result, views},
//!     registry, Query, World,
//! };
//!
//! // Define components.
//! struct Foo(u32);
//! struct Bar(bool);
//! struct Baz(f64);
//!
//! type Registry = registry!(Foo, Bar, Baz);
//!
//! let mut world = World::<Registry>::new();
//! world.insert(entity!(Foo(42), Bar(true), Baz(1.5)));
//!
//! for result!(foo, bar) in world.query(Query::<views!(&mut Foo, &Bar), filter::Has<Baz>>::new()) {
//!     // Do something.
//! }
//! ```
//!
//! [`Component`]: crate::component::Component
//! [`Filter`]: crate::query::filter::Filter
//! [`result!`]: crate::query::result!
//! [`Views`]: crate::query::view::Views
//! [`World`]: crate::world::World

pub mod filter;
pub mod result;
pub mod view;

pub(crate) mod claim;

#[doc(inline)]
pub use result::result;
#[doc(inline)]
pub use view::views;

use core::marker::PhantomData;

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
///     query::{filter, result, views},
///     registry, Query, World,
/// };
///
/// // Define components.
/// struct Foo(u32);
/// struct Bar(bool);
/// struct Baz(f64);
///
/// type Registry = registry!(Foo, Bar, Baz);
///
/// let mut world = World::<Registry>::new();
/// world.insert(entity!(Foo(42), Bar(true), Baz(1.5)));
///
/// for result!(foo, bar) in world.query(Query::<views!(&mut Foo, &Bar), filter::Has<Baz>>::new()) {
///     // Do something.
/// }
/// ```
///
/// [`query()`]: crate::world::World::query()
pub struct Query<V, F = filter::None> {
    view: PhantomData<V>,
    filter: PhantomData<F>,
}

impl<V, F> Query<V, F> {
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
        }
    }
}

impl<V, F> Default for Query<V, F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V, F> Clone for Query<V, F> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<V, F> Copy for Query<V, F> {}
