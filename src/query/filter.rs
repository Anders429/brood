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
//! [`Views`]: trait@crate::query::view::Views
//! [`World`]: crate::world::World

use core::marker::PhantomData;

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
/// [`Views`]: trait@crate::query::view::Views
pub enum None {}

/// Filter based on whether a [`Component`] is present in an entity.
///
/// This filters out any entities which do not have the `Component`. No borrow of the `Component`
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
pub struct Has<Component> {
    component: PhantomData<Component>,
}

/// Filter using the logical inverse of another filter.
///
/// This filters out any entities which would not have been filtered by `Filter`.
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
pub struct Not<Filter> {
    filter: PhantomData<Filter>,
}

/// Filter entities which are filtered by two filters.
///
/// This filter is a logical `and` between two filters `FilterA` and `FilterB`. Any entity filtered
/// out by either filter will be filtered out by the `And` filter.
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
pub struct And<FilterA, FilterB> {
    filter_1: PhantomData<FilterA>,
    filter_2: PhantomData<FilterB>,
}

/// Filter entities which are filtered by one of two filters.
///
/// This filter is a logical `or` between two filters `FilterA` and `FilterB`. Any entity filtered
/// out by both filters will be filtered out by the `Or` filter.
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
pub struct Or<FilterA, FilterB> {
    filter_a: PhantomData<FilterA>,
    filter_b: PhantomData<FilterB>,
}
