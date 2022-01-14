use core::marker::PhantomData;
use rayon::iter::{
    plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer},
    IndexedParallelIterator, ParallelIterator,
};

pub struct RepeatNone<T> {
    inner: PhantomData<T>,
    count: usize,
}

impl<T> RepeatNone<T> {
    pub(crate) fn new(count: usize) -> Self {
        Self {
            inner: PhantomData,
            count,
        }
    }
}

impl<T> ParallelIterator for RepeatNone<T>
where
    T: Send,
{
    type Item = Option<T>;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.count)
    }
}

impl<T> IndexedParallelIterator for RepeatNone<T>
where
    T: Send,
{
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        callback.callback(RepeatNoneProducer {
            inner: self.inner,
            count: self.count,
        })
    }

    fn len(&self) -> usize {
        self.count
    }
}

struct RepeatNoneProducer<T> {
    inner: PhantomData<T>,
    count: usize,
}

impl<T> Producer for RepeatNoneProducer<T>
where
    T: Send,
{
    type Item = Option<T>;
    type IntoIter = RepeatNoneIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        RepeatNoneIter {
            inner: self.inner,
            count: self.count,
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        (
            Self {
                inner: self.inner,
                count: index,
            },
            Self {
                inner: self.inner,
                count: self.count - index,
            },
        )
    }
}

struct RepeatNoneIter<T> {
    inner: PhantomData<T>,
    count: usize,
}

impl<T> Iterator for RepeatNoneIter<T> {
    type Item = Option<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;
            Some(None)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl<T> DoubleEndedIterator for RepeatNoneIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl<T> ExactSizeIterator for RepeatNoneIter<T> {
    #[inline]
    fn len(&self) -> usize {
        self.count
    }
}
