use crate::{
    internal::{archetype, archetype::Archetype},
    registry::Registry,
};
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
    type Item = (archetype::Identifier<R>, &'a Archetype<R>);

    fn next(&mut self) -> Option<Self::Item> {
        self.raw_iter.next().map(|archetype_bucket| {
            let archetype = unsafe { archetype_bucket.as_ref() };
            (unsafe { archetype.identifier() }, archetype)
        })
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
    type Item = (archetype::Identifier<R>, &'a mut Archetype<R>);

    fn next(&mut self) -> Option<Self::Item> {
        self.raw_iter.next().map(|archetype_bucket| {
            let archetype = unsafe { archetype_bucket.as_mut() };
            (unsafe { archetype.identifier() }, archetype)
        })
    }
}
