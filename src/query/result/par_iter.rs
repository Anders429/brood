use crate::{
    archetype::Archetype,
    archetypes,
    hlist::Reshape,
    query::{
        filter::And,
        result::ParResults,
        view::ParViews,
    },
    registry,
    registry::{
        contains::filter::Sealed as ContainsFilterSealed,
        ContainsParQuery,
    },
};
use core::marker::PhantomData;
use rayon::iter::{
    plumbing::{
        Consumer,
        Folder,
        Reducer,
        UnindexedConsumer,
    },
    ParallelIterator,
};

/// A [`ParallelIterator`] over the results of a query.
///
/// Yields results based on the specified `Views` and `Filter`, returning the
/// [`Component`]s viewed. The yielded views will be heterogeneous lists, so the [`result!`] macro
/// is recommended to create identifiers for them.
///
/// This `struct` is created by the [`par_query`] method on [`World`].
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
/// use rayon::iter::ParallelIterator;
///
/// struct Foo(u32);
/// struct Bar(bool);
///
/// type Registry = Registry!(Foo, Bar);
///
/// let mut world = World::<Registry>::new();
/// world.insert(entity!(Foo(42), Bar(true)));
///
/// world
///     .par_query(Query::<Views!(&mut Foo, &Bar)>::new())
///     .iter
///     .for_each(|result!(foo, bar)| {
///         if bar.0 {
///             foo.0 += 1;
///         }
///     });
/// ```
///
/// [`Component`]: crate::component::Component
/// [`ParallelIterator`]: rayon::iter::ParallelIterator
/// [`par_query`]: crate::world::World::par_query()
/// [`result!`]: crate::query::result!
/// [`World`]: crate::world::World
#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub struct ParIter<'a, Registry, Filter, Views, Indices>
where
    Registry: registry::Registry,
{
    archetypes_iter: archetypes::ParIterMut<'a, Registry>,

    filter: PhantomData<Filter>,
    views: PhantomData<Views>,
    indices: PhantomData<Indices>,
}

impl<'a, Registry, Filter, Views, Indices> ParIter<'a, Registry, Filter, Views, Indices>
where
    Registry: registry::Registry,
{
    pub(crate) fn new(archetypes_iter: archetypes::ParIterMut<'a, Registry>) -> Self {
        Self {
            archetypes_iter,

            filter: PhantomData,
            views: PhantomData,
            indices: PhantomData,
        }
    }
}

// SAFETY: This type is safe to send between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<'a, Registry, Filter, Views, Indices> Send
    for ParIter<'a, Registry, Filter, Views, Indices>
where
    Registry: registry::Registry,
{
}

// SAFETY: This type is safe to share between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<'a, Registry, Filter, Views, Indices> Sync
    for ParIter<'a, Registry, Filter, Views, Indices>
where
    Registry: registry::Registry,
{
}

impl<'a, Registry, Filter, Views, Indices> ParallelIterator
    for ParIter<'a, Registry, Filter, Views, Indices>
where
    Views: ParViews<'a>,
    Registry: ContainsParQuery<'a, Filter, Views, Indices>,
{
    type Item = <<Views::ParResults as ParResults>::Iterator as ParallelIterator>::Item;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let consumer = ResultsConsumer::<_, Filter, Views, Indices>::new(consumer);
        self.archetypes_iter.drive_unindexed(consumer)
    }
}

struct ResultsConsumer<Consumer, Filter, Views, Indices> {
    base: Consumer,

    filter: PhantomData<Filter>,
    views: PhantomData<Views>,
    indices: PhantomData<Indices>,
}

impl<Consumer, Filter, Views, Indices> ResultsConsumer<Consumer, Filter, Views, Indices> {
    fn new(base: Consumer) -> Self {
        Self {
            base,

            filter: PhantomData,
            views: PhantomData,
            indices: PhantomData,
        }
    }
}

// SAFETY: This type is safe to send between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<Consumer, Filter, Views, Indices> Send
    for ResultsConsumer<Consumer, Filter, Views, Indices>
{
}

// SAFETY: This type is safe to share between threads, as its mutable views are guaranteed to be
// exclusive.
unsafe impl<Consumer, Filter, Views, Indices> Sync
    for ResultsConsumer<Consumer, Filter, Views, Indices>
{
}

impl<'a, _Consumer, Registry, Filter, Views, Indices> Consumer<&'a mut Archetype<Registry>>
    for ResultsConsumer<_Consumer, Filter, Views, Indices>
where
    _Consumer:
        UnindexedConsumer<<<Views::ParResults as ParResults>::Iterator as ParallelIterator>::Item>,
    Views: ParViews<'a>,
    Registry: ContainsParQuery<'a, Filter, Views, Indices>,
{
    type Folder = ResultsFolder<_Consumer, _Consumer::Result, Filter, Views, Indices>;
    type Reducer = _Consumer::Reducer;
    type Result = _Consumer::Result;

    fn split_at(self, index: usize) -> (Self, Self, _Consumer::Reducer) {
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
            indices: PhantomData,
        }
    }

    fn full(&self) -> bool {
        self.base.full()
    }
}

impl<'a, Consumer, Registry, Filter, Views, Indices> UnindexedConsumer<&'a mut Archetype<Registry>>
    for ResultsConsumer<Consumer, Filter, Views, Indices>
where
    Consumer:
        UnindexedConsumer<<<Views::ParResults as ParResults>::Iterator as ParallelIterator>::Item>,
    Views: ParViews<'a>,
    Registry: ContainsParQuery<'a, Filter, Views, Indices>,
{
    fn split_off_left(&self) -> Self {
        ResultsConsumer::new(self.base.split_off_left())
    }

    fn to_reducer(&self) -> Self::Reducer {
        self.base.to_reducer()
    }
}

struct ResultsFolder<Consumer, Previous, Filter, Views, Indices> {
    base: Consumer,
    previous: Option<Previous>,

    filter: PhantomData<Filter>,
    views: PhantomData<Views>,
    indices: PhantomData<Indices>,
}

impl<'a, Consumer, Registry, Filter, Views, Indices> Folder<&'a mut Archetype<Registry>>
    for ResultsFolder<Consumer, Consumer::Result, Filter, Views, Indices>
where
    Consumer:
        UnindexedConsumer<<<Views::ParResults as ParResults>::Iterator as ParallelIterator>::Item>,
    Registry: ContainsParQuery<'a, Filter, Views, Indices>,
    Views: ParViews<'a>,
{
    type Result = Consumer::Result;

    fn consume(self, archetype: &'a mut Archetype<Registry>) -> Self {
        // SAFETY: The `R` on which `filter()` is called is the same `R` over which the identifier
        // is generic over. Additionally, the identifier reference created here will not outlive
        // `archetype`.
        if unsafe {
            <Registry as ContainsFilterSealed<
                And<Views, Filter>,
                And<Registry::ViewsFilterIndices, Registry::FilterIndices>,
            >>::filter(archetype.identifier())
        } {
            let consumer = self.base.split_off_left();
            let result =
                // SAFETY: Each component viewed by `V` is guaranteed to be within the `archetype`
                // since the `filter` function in the if-statement returned `true`.
                unsafe { archetype.par_view::<Views, _, _, _>() }.reshape().into_parallel_iterator().drive_unindexed(consumer);

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
                indices: self.indices,
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
