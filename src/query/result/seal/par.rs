use crate::query::view;
use rayon::{iter, iter::IndexedParallelIterator};

pub trait ParResults {
    type View: Send;
    type Iterator: IndexedParallelIterator<Item = Self::View>;

    fn into_parallel_iterator(self) -> Self::Iterator;
}

impl ParResults for iter::RepeatN<view::Null> {
    type View = view::Null;
    type Iterator = Self;

    fn into_parallel_iterator(self) -> Self::Iterator {
        self
    }
}

impl<C, I, R> ParResults for (I, R)
where
    C: Send,
    I: IndexedParallelIterator<Item = C>,
    R: ParResults,
{
    type View = (C, R::View);
    type Iterator = iter::Zip<I, R::Iterator>;

    fn into_parallel_iterator(self) -> Self::Iterator {
        self.0.zip(self.1.into_parallel_iterator())
    }
}
