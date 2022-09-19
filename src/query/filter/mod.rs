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

use crate::{component::Component, entity, query::view, registry, registry::Registry};
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
pub trait Filter<R, I>: Seal<R, I>
where
    R: Registry,
{
}

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
pub enum None {}

impl<R> Filter<R, None> for None where R: Registry {}

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

impl<C, I, R> Filter<R, I> for Has<C>
where
    C: Component,
    R: Registry + registry::Filter<C, I>,
{
}

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
pub struct Not<F> {
    filter: PhantomData<F>,
}

impl<F, I, R> Filter<R, Not<I>> for Not<F>
where
    F: Filter<R, I>,
    R: Registry,
{
}

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
pub struct And<F1, F2> {
    filter_1: PhantomData<F1>,
    filter_2: PhantomData<F2>,
}

impl<F1, F2, I, J, R> Filter<R, And<I, J>> for And<F1, F2>
where
    F1: Filter<R, I>,
    F2: Filter<R, J>,
    R: Registry,
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
pub struct Or<F1, F2> {
    filter_1: PhantomData<F1>,
    filter_2: PhantomData<F2>,
}

impl<F1, F2, I, J, R> Filter<R, Or<I, J>> for Or<F1, F2>
where
    F1: Filter<R, I>,
    F2: Filter<R, J>,
    R: Registry,
{
}

impl<C, I, R> Filter<R, I> for &C
where
    C: Component,
    R: Registry + registry::Filter<C, I>,
{
}

impl<C, I, R> Filter<R, I> for &mut C
where
    C: Component,
    R: Registry + registry::Filter<C, I>,
{
}

impl<C, R> Filter<R, None> for Option<&C>
where
    C: Component,
    R: Registry,
{
}

impl<C, R> Filter<R, None> for Option<&mut C>
where
    C: Component,
    R: Registry,
{
}

impl<R> Filter<R, None> for entity::Identifier where R: Registry {}

impl<R> Filter<R, None> for view::Null where R: Registry {}

impl<I, J, R, V, W> Filter<R, And<I, J>> for (V, W)
where
    R: Registry,
    V: Filter<R, I>,
    W: Filter<R, J>,
{
}
