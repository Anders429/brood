use crate::{
    archetypes,
    query::{
        filter::And,
        result::{
            Reshape,
            Results,
        },
        view,
    },
    registry,
    registry::{
        contains::filter::Sealed as ContainsFilterSealed,
        ContainsQuery,
    },
};
use core::{
    iter::FusedIterator,
    marker::PhantomData,
};

/// An [`Iterator`] over the results of a query.
///
/// Yields results based on the specified [`Views`] `V` and [`Filter`] `F`, returning the
/// [`Component`]s viewed. The entities iterated are not in any specified order. The yielded views
/// will be heterogeneous lists, so the [`result!`] macro is recommended to create identifiers for
/// them.
///
/// This `struct` is created by the [`query`] method on [`World`].
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
/// struct Foo(u32);
/// struct Bar(bool);
///
/// type Registry = Registry!(Foo, Bar);
///
/// let mut world = World::<Registry>::new();
/// world.insert(entity!(Foo(42), Bar(true)));
///
/// for result!(foo, bar) in world.query(Query::<Views!(&mut Foo, &Bar)>::new()).iter {
///     if bar.0 {
///         foo.0 += 1;
///     }
/// }
/// ```
///
/// [`Component`]: crate::component::Component
/// [`Filter`]: crate::query::filter::Filter
/// [`query`]: crate::world::World::query()
/// [`result!`]: crate::query::result!
/// [`Views`]: trait@crate::query::view::Views
/// [`World`]: crate::world::World
pub struct Iter<'a, Registry, Filter, Views, Indices>
where
    Registry: registry::Registry,
    Views: view::Views<'a>,
{
    archetypes_iter: archetypes::IterMut<'a, Registry>,

    current_results_iter: Option<<Views::Results as Results>::Iterator>,

    filter: PhantomData<Filter>,
    indices: PhantomData<Indices>,
}

impl<'a, Registry, Filter, Views, Indices> Iter<'a, Registry, Filter, Views, Indices>
where
    Registry: registry::Registry,
    Views: view::Views<'a>,
{
    pub(crate) fn new(archetypes_iter: archetypes::IterMut<'a, Registry>) -> Self {
        Self {
            archetypes_iter,

            current_results_iter: None,

            filter: PhantomData,
            indices: PhantomData,
        }
    }
}

impl<'a, Registry, Filter, Views, Indices> Iterator for Iter<'a, Registry, Filter, Views, Indices>
where
    Views: view::Views<'a>,
    Registry: ContainsQuery<'a, Filter, Views, Indices>,
{
    type Item = Views;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut results) = self.current_results_iter {
                if let result @ Some(_) = results.next() {
                    return result;
                }
            }
            let archetype = self.archetypes_iter.find(|archetype| {
                // SAFETY: The `R` on which `filter()` is called is the same `R` over which the
                // identifier is generic over. Additionally, the identifier reference created here
                // will not outlive `archetype`.
                unsafe {
                    <Registry as ContainsFilterSealed<
                        And<Views, Filter>,
                        And<Registry::ViewsFilterIndices, Registry::FilterIndices>,
                    >>::filter(archetype.identifier())
                }
            })?;
            self.current_results_iter = Some(
                // SAFETY: Each component viewed by `V` is guaranteed to be within the `archetype`,
                // since the archetype was not removed by the `find()` method above which filters
                // out archetypes that do not contain the viewed components.
                unsafe {
                    archetype.view::<Views, (
                        Registry::ViewsContainments,
                        Registry::ViewsIndices,
                        Registry::ViewsCanonicalContainments,
                    )>()
                }
                .reshape()
                .into_iterator(),
            );
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.current_results_iter.as_ref().map_or(
            (0, Some(0)),
            <Views::Results as Results>::Iterator::size_hint,
        );
        match (self.archetypes_iter.size_hint(), high) {
            ((0, Some(0)), Some(_)) => (low, high),
            _ => (low, None),
        }
    }

    #[inline]
    fn fold<A, Fold>(self, mut init: A, mut fold: Fold) -> A
    where
        Fold: FnMut(A, Self::Item) -> A,
    {
        if let Some(results) = self.current_results_iter {
            init = results.fold(init, &mut fold);
        }

        self.archetypes_iter.fold(init, |acc, archetype| {
            // SAFETY: The `R` on which `filter()` is called is the same `R` over which the
            // identifier is generic over. Additionally, the identifier reference created here will
            // not outlive `archetype`.
            if unsafe {
                <Registry as ContainsFilterSealed<
                    And<Views, Filter>,
                    And<Registry::ViewsFilterIndices, Registry::FilterIndices>,
                >>::filter(archetype.identifier())
            } {
                // SAFETY: Each component viewed by `V` is guaranteed to be within the `archetype`
                // since the `filter` function in the if-statement returned `true`.
                unsafe {
                    archetype.view::<Views, (
                        Registry::ViewsContainments,
                        Registry::ViewsIndices,
                        Registry::ViewsCanonicalContainments,
                    )>()
                }
                .reshape()
                .into_iterator()
                .fold(acc, &mut fold)
            } else {
                acc
            }
        })
    }
}

impl<'a, Registry, Filter, Views, Indices> FusedIterator
    for Iter<'a, Registry, Filter, Views, Indices>
where
    Views: view::Views<'a>,
    Registry: ContainsQuery<'a, Filter, Views, Indices>,
{
}

// SAFETY: This type is safe to send between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<'a, Registry, Filter, Views, Indices> Send
    for Iter<'a, Registry, Filter, Views, Indices>
where
    Registry: registry::Registry,
    Views: view::Views<'a>,
{
}
