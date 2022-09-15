use crate::{
    archetype::Archetype,
    archetypes,
    query::{
        filter::{And, Filter, Seal},
        result::{ParResults, Reshape},
        view::{ParViews, ParViewsSeal},
    },
    registry::{ContainsParViews, Registry},
};
use core::marker::PhantomData;
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
///     registry, Query, World,
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
///     .par_query(Query::<views!(&mut Foo, &Bar), filter::None>::new())
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
#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub struct ParIter<'a, R, F, FI, V, VI, P, I, Q>
where
    R: Registry,
    F: Filter<R, FI>,
    V: ParViews<'a> + Filter<R, VI>,
{
    archetypes_iter: archetypes::ParIterMut<'a, R>,

    filter: PhantomData<F>,
    views: PhantomData<V>,
    filter_indices: PhantomData<FI>,
    view_filter_indices: PhantomData<VI>,
    view_containments: PhantomData<P>,
    view_indices: PhantomData<I>,
    reshape_indices: PhantomData<Q>,
}

impl<'a, R, F, FI, V, VI, P, I, Q> ParIter<'a, R, F, FI, V, VI, P, I, Q>
where
    R: Registry,
    F: Filter<R, FI>,
    V: ParViews<'a> + Filter<R, VI>,
{
    pub(crate) fn new(archetypes_iter: archetypes::ParIterMut<'a, R>) -> Self {
        Self {
            archetypes_iter,

            filter: PhantomData,
            views: PhantomData,
            filter_indices: PhantomData,
            view_filter_indices: PhantomData,
            view_containments: PhantomData,
            view_indices: PhantomData,
            reshape_indices: PhantomData,
        }
    }
}

// SAFETY: This type is safe to send between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<'a, R, F, FI, V, VI, P, I, Q> Send for ParIter<'a, R, F, FI, V, VI, P, I, Q>
where
    R: Registry,
    F: Filter<R, FI>,
    V: ParViews<'a> + Filter<R, VI>,
{
}

// SAFETY: This type is safe to share between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<'a, R, F, FI, V, VI, P, I, Q> Sync for ParIter<'a, R, F, FI, V, VI, P, I, Q>
where
    R: Registry,
    F: Filter<R, FI>,
    V: ParViews<'a> + Filter<R, VI>,
{
}

impl<'a, R, F, FI, V, VI, P, I, Q> ParallelIterator for ParIter<'a, R, F, FI, V, VI, P, I, Q>
where
    R: Registry + 'a,
    F: Filter<R, FI>,
    V: ParViews<'a> + Filter<R, VI>,
    R::Viewable: ContainsParViews<'a, V, P, I, Q>,
{
    type Item = <<V as ParViewsSeal<'a>>::ParResults as ParResults>::View;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let consumer = ResultsConsumer::<_, F, FI, V, VI, P, I, Q>::new(consumer);
        self.archetypes_iter.drive_unindexed(consumer)
    }
}

struct ResultsConsumer<C, F, FI, V, VI, P, I, Q> {
    base: C,

    filter: PhantomData<F>,
    views: PhantomData<V>,
    filter_indices: PhantomData<FI>,
    view_filter_indices: PhantomData<VI>,
    view_containments: PhantomData<P>,
    view_indices: PhantomData<I>,
    reshape_indices: PhantomData<Q>,
}

impl<C, F, FI, V, VI, P, I, Q> ResultsConsumer<C, F, FI, V, VI, P, I, Q> {
    fn new(base: C) -> Self {
        Self {
            base,

            filter: PhantomData,
            views: PhantomData,
            filter_indices: PhantomData,
            view_filter_indices: PhantomData,
            view_containments: PhantomData,
            view_indices: PhantomData,
            reshape_indices: PhantomData,
        }
    }
}

// SAFETY: This type is safe to send between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<C, F, FI, V, VI, P, I, Q> Send for ResultsConsumer<C, F, FI, V, VI, P, I, Q> {}

// SAFETY: This type is safe to share between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<C, F, FI, V, VI, P, I, Q> Sync for ResultsConsumer<C, F, FI, V, VI, P, I, Q> {}

impl<'a, C, R, F, FI, V, VI, P, I, Q> Consumer<&'a mut Archetype<R>>
    for ResultsConsumer<C, F, FI, V, VI, P, I, Q>
where
    C: UnindexedConsumer<<<V::ParResults as ParResults>::Iterator as ParallelIterator>::Item>,
    R: Registry,
    F: Filter<R, FI>,
    V: ParViews<'a> + Filter<R, VI>,
    R::Viewable: ContainsParViews<'a, V, P, I, Q>,
{
    type Folder = ResultsFolder<C, C::Result, F, FI, V, VI, P, I, Q>;
    type Reducer = C::Reducer;
    type Result = C::Result;

    fn split_at(self, index: usize) -> (Self, Self, C::Reducer) {
        let (left, right, reducer) = self.base.split_at(index);
        (
            ResultsConsumer::new(left),
            ResultsConsumer::new(right),
            reducer,
        )
    }

    fn into_folder(self) -> Self::Folder {
        ResultsFolder {
            base: self.base,
            previous: None,

            filter: PhantomData,
            views: PhantomData,
            filter_indices: PhantomData,
            view_filter_indices: PhantomData,
            view_containments: PhantomData,
            view_indices: PhantomData,
            reshape_indices: PhantomData,
        }
    }

    fn full(&self) -> bool {
        self.base.full()
    }
}

impl<'a, C, R, F, FI, V, VI, P, I, Q> UnindexedConsumer<&'a mut Archetype<R>>
    for ResultsConsumer<C, F, FI, V, VI, P, I, Q>
where
    C: UnindexedConsumer<<<V::ParResults as ParResults>::Iterator as ParallelIterator>::Item>,
    R: Registry,
    F: Filter<R, FI>,
    V: ParViews<'a> + Filter<R, VI>,
    R::Viewable: ContainsParViews<'a, V, P, I, Q>,
{
    fn split_off_left(&self) -> Self {
        ResultsConsumer::new(self.base.split_off_left())
    }

    fn to_reducer(&self) -> Self::Reducer {
        self.base.to_reducer()
    }
}

struct ResultsFolder<C, P, F, FI, V, VI, P_, I, Q> {
    base: C,
    previous: Option<P>,

    filter: PhantomData<F>,
    views: PhantomData<V>,
    filter_indices: PhantomData<FI>,
    view_filter_indices: PhantomData<VI>,
    view_containments: PhantomData<P_>,
    view_indices: PhantomData<I>,
    reshape_indices: PhantomData<Q>,
}

impl<'a, C, R, F, FI, V, VI, P, I, Q> Folder<&'a mut Archetype<R>>
    for ResultsFolder<C, C::Result, F, FI, V, VI, P, I, Q>
where
    C: UnindexedConsumer<<<V::ParResults as ParResults>::Iterator as ParallelIterator>::Item>,
    R: Registry,
    R::Viewable: ContainsParViews<'a, V, P, I, Q>,
    F: Filter<R, FI>,
    V: ParViews<'a> + Filter<R, VI>,
{
    type Result = C::Result;

    fn consume(self, archetype: &'a mut Archetype<R>) -> Self {
        if And::<V, F>::filter(
            // SAFETY: This identifier reference will not outlive `archetype`.
            unsafe { archetype.identifier() },
        ) {
            let consumer = self.base.split_off_left();
            let result =
                // SAFETY: Each component viewed by `V` is guaranteed to be within the `archetype`
                // since the `filter` function in the if-statement returned `true`.
                unsafe { archetype.par_view::<V, _, _, _>() }.reshape().into_parallel_iterator().drive_unindexed(consumer);

            let previous = match self.previous {
                None => Some(result),
                Some(previous) => {
                    let reducer = self.base.to_reducer();
                    Some(reducer.reduce(previous, result))
                }
            };

            ResultsFolder {
                base: self.base,
                previous,

                filter: self.filter,
                views: self.views,
                filter_indices: self.filter_indices,
                view_filter_indices: self.view_filter_indices,
                view_containments: self.view_containments,
                view_indices: self.view_indices,
                reshape_indices: self.reshape_indices,
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
