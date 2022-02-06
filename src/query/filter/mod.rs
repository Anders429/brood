//! Filters for entities.
//!
//! `Filter`s are used to filter results when querying entities. Along with [`Views`], they are
//! what make up a query over a [`World`]. They provide the ability to query based on a
//! [`Component`] without actually having to borrow the `Component`'s value.
//!
//! Note that some `Views` provide implicit `Filter`s. Specifically, non-optional views over a
//! `Component` `C` (`&C` and `&mut C`) both implicitly [`And`] a [`Has<C>`] `Filter` with other
//! `Filter`s.
//!
//! [`And`]: crate::query::filter::And
//! [`Component`]: crate::component::Component
//! [`Has<C>`]: crate::query::filter::Has
//! [`Views`]: crate::query::view::Views
//! [`World`]: crate::world::World

mod seal;

pub(crate) use seal::Seal;

use crate::{
    component::Component,
    entity,
    query::{
        view,
        view::{View, Views},
    },
};
use core::marker::PhantomData;

/// A filter for entities.
///
/// `Filter`s are used to filter results when querying entities. Along with [`Views`], they are
/// what make up a query over a [`World`]. They provide the ability to query based on a
/// [`Component`] without actually having to borrow the `Component`'s value.
///
/// Note that some `Views` provide implicit `Filter`s. Specifically, non-optional views over a
/// `Component` `C` (`&C` and `&mut C`) both implicitly [`And`] a [`Has<C>`] `Filter` with other
/// `Filter`s.
///
/// [`And`]: crate::query::filter::And
/// [`Component`]: crate::component::Component
/// [`Has<C>`]: crate::query::filter::Has
/// [`Views`]: crate::query::view::Views
/// [`World`]: crate::world::World
pub trait Filter: Seal {}

/// An empty filter.
///
/// This filter is used when a filter is required but nothing is needed to be filtered. For
/// example, some queries don't need any additional filtering beyond the filtering already provided
/// by their [`Views`]. This filter can be provided in those cases.
///
/// # Example
/// ``` rust
/// use brood::query::filter;
///
/// type NoFilter = filter::None;
/// ```
///
/// [`Views`]: crate::query::view::Views
pub struct None;

impl Filter for None {}

/// Filter based on whether a [`Component`] is present in an entity.
///
/// This filters out any entities which do not have the `Component` `C`. No borrow of the value `C`
/// from the entity is required.
///
/// # Example'
/// ``` rust
/// use brood::query::filter;
///
/// // Define a component.
/// struct Foo(usize);
///
/// // Define a filter for the component above.
/// type HasFoo = filter::Has<Foo>;
/// ```
///
/// [`Component`]: crate::component::Component
pub struct Has<C>
where
    C: Component,
{
    component: PhantomData<C>,
}

impl<C> Filter for Has<C> where C: Component {}

/// Filter using the logical inverse of another [`Filter`].
///
/// This filters out any entities which would not have been filtered by the `Filter` `F`.
///
/// # Example
/// ``` rust
/// use brood::query::filter;
///
/// // Define a component.
/// struct Foo(usize);
///
/// // Define a filter for the component above.
/// type HasFoo = filter::Has<Foo>;
///
/// // Define a component that is the inverse of the filter above.
/// type DoesNotHaveFoo = filter::Not<HasFoo>;
/// ```
///
/// [`Filter`]: crate::query::filter::Filter
pub struct Not<F>
where
    F: Filter,
{
    filter: PhantomData<F>,
}

impl<F> Filter for Not<F> where F: Filter {}

/// Filter entities which are filtered by two [`Filter`]s.
///
/// This filter is a logical `and` between two `Filter`s `F1` and `F2`. Any entity filtered out by
/// either `Filter` will be filtered out by the `And` filter.
///
/// # Example
/// ``` rust
/// use brood::query::filter;
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// // Define filters based on the above components.
/// type HasFoo = filter::Has<Foo>;
/// type HasBar = filter::Has<Bar>;
///
/// // Define a filter using a combination of the above filters.
/// type HasFooAndBar = filter::And<HasFoo, HasBar>;
/// ```
///
/// [`Filter`]: crate::query::filter::Filter
pub struct And<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    filter_1: PhantomData<F1>,
    filter_2: PhantomData<F2>,
}

impl<F1, F2> Filter for And<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
}

/// Filter entities which are filtered by one of two [`Filter`]s.
///
/// This filter is a logical `or` between two `Filter`s `F1` and `F2`. Any entity filtered out by
/// both `Filter`s will be filtered out by the `Or` filter.
///
/// # Example
/// ``` rust
/// use brood::query::filter;
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// // Define filters based on the above components.
/// type HasFoo = filter::Has<Foo>;
/// type HasBar = filter::Has<Bar>;
///
/// // Define a filter using a combination of the above filters.
/// type HasFooOrBar = filter::Or<HasFoo, HasBar>;
/// ```
///
/// [`Filter`]: crate::query::filter::Filter
pub struct Or<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    filter_1: PhantomData<F1>,
    filter_2: PhantomData<F2>,
}

impl<F1, F2> Filter for Or<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
}

impl<C> Filter for &C where C: Component {}

impl<C> Filter for &mut C where C: Component {}

impl<C> Filter for Option<&C> where C: Component {}

impl<C> Filter for Option<&mut C> where C: Component {}

impl Filter for entity::Identifier {}

impl Filter for view::Null {}

impl<'a, V, W> Filter for (V, W)
where
    V: View<'a>,
    W: Views<'a>,
{
}
