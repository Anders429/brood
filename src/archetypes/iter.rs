use crate::{archetype::Archetype, registry::Registry};
use core::marker::PhantomData;
use hashbrown::raw::RawIter;

pub(crate) struct Iter<'a, R>
where
    R: Registry,
{
    lifetime: PhantomData<&'a ()>,

    raw_iter: RawIter<Archetype<R>>,
}

impl<'a, R> Iter<'a, R>
where
    R: Registry,
{
    pub(super) fn new(raw_iter: RawIter<Archetype<R>>) -> Self {
        Self {
            lifetime: PhantomData,

            raw_iter,
        }
    }
}

impl<'a, R> Iterator for Iter<'a, R>
where
    R: Registry + 'a,
{
    type Item = &'a Archetype<R>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw_iter.next().map(|archetype_bucket|
            unsafe { archetype_bucket.as_ref() }
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw_iter.size_hint()
    }
}

pub(crate) struct IterMut<'a, R>
where
    R: Registry,
{
    lifetime: PhantomData<&'a ()>,

    raw_iter: RawIter<Archetype<R>>,
}

impl<'a, R> IterMut<'a, R>
where
    R: Registry,
{
    pub(super) fn new(raw_iter: RawIter<Archetype<R>>) -> Self {
        Self {
            lifetime: PhantomData,

            raw_iter,
        }
    }
}

impl<'a, R> Iterator for IterMut<'a, R>
where
    R: Registry + 'a,
{
    type Item = &'a mut Archetype<R>;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw_iter.next().map(|archetype_bucket|
            unsafe { archetype_bucket.as_mut() }
        )
    }
}
