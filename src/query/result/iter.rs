use crate::{
    archetypes,
    query::{
        filter::{And, Filter, Seal},
        result::Results,
        view::Views,
    },
    registry::Registry,
};
use core::{iter::FusedIterator, marker::PhantomData};

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
///     query::{filter, result, views},
///     registry, World,
/// };
///
/// struct Foo(u32);
/// struct Bar(bool);
///
/// type Registry = registry!(Foo, Bar);
///
/// let mut world = World::<Registry>::new();
/// world.insert(entity!(Foo(42), Bar(true)));
///
/// for result!(foo, bar) in world.query::<views!(&mut Foo, &Bar), filter::None, _, _, _, _>() {
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
/// [`Views`]: crate::query::view::Views
/// [`World`]: crate::world::World
pub struct Iter<'a, R, F, FI, V, VI>
where
    R: Registry,
    F: Filter<R, FI>,
    V: Views<'a> + Filter<R, VI>,
{
    archetypes_iter: archetypes::IterMut<'a, R>,

    current_results_iter: Option<<V::Results as Results>::Iterator>,

    filter: PhantomData<F>,
    filter_indices: PhantomData<FI>,
    view_indices: PhantomData<VI>,
}

impl<'a, R, F, FI, V, VI> Iter<'a, R, F, FI, V, VI>
where
    R: Registry,
    F: Filter<R, FI>,
    V: Views<'a> + Filter<R, VI>,
{
    pub(crate) fn new(archetypes_iter: archetypes::IterMut<'a, R>) -> Self {
        Self {
            archetypes_iter,

            current_results_iter: None,

            filter: PhantomData,
            filter_indices: PhantomData,
            view_indices: PhantomData,
        }
    }
}

impl<'a, R, F, FI, V, VI> Iterator for Iter<'a, R, F, FI, V, VI>
where
    R: Registry + 'a,
    F: Filter<R, FI>,
    V: Views<'a> + Filter<R, VI>,
{
    type Item = V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut results) = self.current_results_iter {
                if let result @ Some(_) = results.next() {
                    return result;
                }
            }
            let archetype = self.archetypes_iter.find(|archetype| {
                And::<V, F>::filter(
                    // SAFETY: This identifier reference will not outlive `archetype`.
                    unsafe { archetype.identifier() },
                )
            })?;
            self.current_results_iter = Some(
                // SAFETY: Each component viewed by `V` is guaranteed to be within the `archetype`,
                // since the archetype was not removed by the `find()` method above which filters
                // out archetypes that do not contain the viewed components.
                unsafe { archetype.view::<V>() }.into_iterator(),
            );
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self
            .current_results_iter
            .as_ref()
            .map_or((0, Some(0)), <V::Results as Results>::Iterator::size_hint);
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
            if And::<V, F>::filter(
                // SAFETY: This identifier reference will not outlive `archetype`.
                unsafe { archetype.identifier() },
            ) {
                // SAFETY: Each component viewed by `V` is guaranteed to be within the `archetype`
                // since the `filter` function in the if-statement returned `true`.
                unsafe { archetype.view::<V>() }.into_iterator().fold(acc, &mut fold)
            } else {
                acc
            }
        })
    }
}

impl<'a, R, F, FI, V, VI> FusedIterator for Iter<'a, R, F, FI, V, VI>
where
    R: Registry + 'a,
    F: Filter<R, FI>,
    V: Views<'a> + Filter<R, VI>,
{
}

// SAFETY: This type is safe to send between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<'a, R, F, FI, V, VI> Send for Iter<'a, R, F, FI, V, VI>
where
    R: Registry + 'a,
    F: Filter<R, FI>,
    V: Views<'a> + Filter<R, VI>,
{
}
