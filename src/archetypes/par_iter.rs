use crate::{
    archetype::Archetype,
    archetypes::Archetypes,
    registry::Registry,
};
use core::marker::PhantomData;
use hashbrown::raw::rayon::RawParIter;
use rayon::iter::{
    plumbing::UnindexedConsumer,
    ParallelIterator,
};

#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub(crate) struct ParIterMut<'a, R>
where
    R: Registry,
{
    lifetime: PhantomData<&'a ()>,

    raw_iter: RawParIter<Archetype<R>>,
}

impl<R> ParIterMut<'_, R>
where
    R: Registry,
{
    fn new(raw_iter: RawParIter<Archetype<R>>) -> Self {
        Self {
            lifetime: PhantomData,

            raw_iter,
        }
    }
}

impl<'a, R> ParallelIterator for ParIterMut<'a, R>
where
    R: Registry,
{
    type Item = &'a mut Archetype<R>;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.raw_iter
            .map(|archetype_bucket| {
                // SAFETY: The reference to the archetype stored in this bucket is guaranteed to be
                // unique.
                unsafe { archetype_bucket.as_mut() }
            })
            .drive_unindexed(consumer)
    }
}

impl<R> Archetypes<R>
where
    R: Registry,
{
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    pub(crate) fn par_iter_mut(&mut self) -> ParIterMut<R> {
        ParIterMut::new(
            // SAFETY: The `ParIterMut` containing this `RawIter` is guaranteed to not outlive
            // `self`.
            unsafe { self.raw_archetypes.par_iter() },
        )
    }
}
