use crate::{
    archetype::Archetype,
    archetypes,
    query::{
        filter::{And, Filter, Seal},
        view::ParViews,
    },
    registry::Registry,
};
use core::{any::TypeId, marker::PhantomData};
use hashbrown::HashMap;
use rayon::iter::{
    plumbing::{Consumer, Folder, Reducer, UnindexedConsumer},
    ParallelIterator,
};

/// A [`ParallelIterator`] over the results of a query.
///
/// Yields results based on the specified [`ParViews`] `V` and [`Filter`] `F`, return the
/// [`Component`]s viewed. The yielded views will be heterogeneous lists, so the [`result!`] macro
/// is recommended to create identifiers for them.
///
/// This `struct` is created by the [`par_query`] method on [`World`].
///
/// # Example
/// ``` rust
/// use brood::{
///     entity,
///     query::{filter, result, views},
///     registry, World,
/// };
/// use rayon::iter::ParallelIterator;
///
/// struct Foo(u32);
/// struct Bar(bool);
///
/// type Registry = registry!(Foo, Bar);
///
/// let mut world = World::<Registry>::new();
/// world.insert(entity!(Foo(42), Bar(true)));
///
/// world
///     .par_query::<views!(&mut Foo, &Bar), filter::None>()
///     .for_each(|result!(foo, bar)| {
///         if bar.0 {
///             foo.0 += 1;
///         }
///     });
/// ```
///
/// [`Component`]: crate::component::Component
/// [`Filter`]: crate::query::filter::Filter
/// [`ParallelIterator`]: rayon::iter::ParallelIterator
/// [`par_query`]: crate::world::World::par_query()
/// [`ParViews`]: crate::query::view::ParViews
/// [`result!`]: crate::query::result!
/// [`World`]: crate::world::World
#[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
pub struct ParIter<'a, R, F, V>
where
    R: Registry,
    F: Filter,
    V: ParViews<'a>,
{
    archetypes_iter: archetypes::ParIterMut<'a, R>,

    component_map: &'a HashMap<TypeId, usize>,

    filter: PhantomData<F>,
    views: PhantomData<V>,
}

impl<'a, R, F, V> ParIter<'a, R, F, V>
where
    R: Registry,
    F: Filter,
    V: ParViews<'a>,
{
    pub(crate) fn new(
        archetypes_iter: archetypes::ParIterMut<'a, R>,
        component_map: &'a HashMap<TypeId, usize>,
    ) -> Self {
        Self {
            archetypes_iter,

            component_map,

            filter: PhantomData,
            views: PhantomData,
        }
    }
}

// SAFETY: This type is safe to send between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<'a, R, F, V> Send for ParIter<'a, R, F, V>
where
    R: Registry,
    F: Filter,
    V: ParViews<'a>,
{
}

// SAFETY: This type is safe to share between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<'a, R, F, V> Sync for ParIter<'a, R, F, V>
where
    R: Registry,
    F: Filter,
    V: ParViews<'a>,
{
}

impl<'a, R, F, V> ParallelIterator for ParIter<'a, R, F, V>
where
    R: Registry + 'a,
    F: Filter,
    V: ParViews<'a>,
{
    type Item = <V::ParResults as ParallelIterator>::Item;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let consumer = ResultsConsumer::<_, F, V>::new(consumer, self.component_map);
        self.archetypes_iter.drive_unindexed(consumer)
    }
}

struct ResultsConsumer<'a, C, F, V> {
    base: C,
    component_map: &'a HashMap<TypeId, usize>,

    filter: PhantomData<F>,
    views: PhantomData<V>,
}

impl<'a, C, F, V> ResultsConsumer<'a, C, F, V> {
    fn new(base: C, component_map: &'a HashMap<TypeId, usize>) -> Self {
        Self {
            base,
            component_map,

            filter: PhantomData,
            views: PhantomData,
        }
    }
}

// SAFETY: This type is safe to send between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<C, F, V> Send for ResultsConsumer<'_, C, F, V> {}

// SAFETY: This type is safe to share between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<C, F, V> Sync for ResultsConsumer<'_, C, F, V> {}

impl<'a, C, R, F, V> Consumer<&'a mut Archetype<R>> for ResultsConsumer<'a, C, F, V>
where
    C: UnindexedConsumer<<V::ParResults as ParallelIterator>::Item>,
    R: Registry,
    F: Filter,
    V: ParViews<'a>,
{
    type Folder = ResultsFolder<'a, C, C::Result, F, V>;
    type Reducer = C::Reducer;
    type Result = C::Result;

    fn split_at(self, index: usize) -> (Self, Self, C::Reducer) {
        let (left, right, reducer) = self.base.split_at(index);
        (
            ResultsConsumer::new(left, self.component_map),
            ResultsConsumer::new(right, self.component_map),
            reducer,
        )
    }

    fn into_folder(self) -> Self::Folder {
        ResultsFolder {
            base: self.base,
            component_map: self.component_map,
            previous: None,

            filter: PhantomData,
            views: PhantomData,
        }
    }

    fn full(&self) -> bool {
        self.base.full()
    }
}

impl<'a, C, R, F, V> UnindexedConsumer<&'a mut Archetype<R>> for ResultsConsumer<'a, C, F, V>
where
    C: UnindexedConsumer<<V::ParResults as ParallelIterator>::Item>,
    R: Registry,
    F: Filter,
    V: ParViews<'a>,
{
    fn split_off_left(&self) -> Self {
        ResultsConsumer::new(self.base.split_off_left(), self.component_map)
    }

    fn to_reducer(&self) -> Self::Reducer {
        self.base.to_reducer()
    }
}

struct ResultsFolder<'a, C, P, F, V> {
    base: C,
    component_map: &'a HashMap<TypeId, usize>,
    previous: Option<P>,

    filter: PhantomData<F>,
    views: PhantomData<V>,
}

impl<'a, C, R, F, V> Folder<&'a mut Archetype<R>> for ResultsFolder<'a, C, C::Result, F, V>
where
    C: UnindexedConsumer<<V::ParResults as ParallelIterator>::Item>,
    R: Registry,
    F: Filter,
    V: ParViews<'a>,
{
    type Result = C::Result;

    fn consume(self, archetype: &'a mut Archetype<R>) -> Self {
        if
        // SAFETY: `self.component_map` contains an entry for each `TypeId<C>` per component `C` in
        // the registry `R`.
        unsafe { And::<V, F>::filter(archetype.identifier(), self.component_map) } {
            let consumer = self.base.split_off_left();
            let result =
                // SAFETY: Each component viewed by `V` is guaranteed to be within the `archetype`
                // since the `filter` function in the if-statement returned `true`.
                unsafe { archetype.par_view::<V>() }.drive_unindexed(consumer);

            let previous = match self.previous {
                None => Some(result),
                Some(previous) => {
                    let reducer = self.base.to_reducer();
                    Some(reducer.reduce(previous, result))
                }
            };

            ResultsFolder {
                base: self.base,
                component_map: self.component_map,
                previous,

                filter: self.filter,
                views: self.views,
            }
        } else {
            self
        }
    }

    fn complete(self) -> Self::Result {
        match self.previous {
            Some(previous) => previous,
            None => self.base.into_folder().complete(),
        }
    }

    fn full(&self) -> bool {
        self.base.full()
    }
}
