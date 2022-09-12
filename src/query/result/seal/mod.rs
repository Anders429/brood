#[cfg(feature = "rayon")]
mod par;

#[cfg(feature = "rayon")]
pub(crate) use par::ParResults;

use crate::query::view;
use core::iter;

pub trait Results {
    type View;
    type Iterator: Iterator<Item = Self::View>;

    fn into_iterator(self) -> Self::Iterator;
}

impl Results for iter::Repeat<view::Null> {
    type View = view::Null;
    type Iterator = Self;

    fn into_iterator(self) -> Self::Iterator {
        self
    }
}

impl<C, I, R> Results for (I, R)
where
    I: Iterator<Item = C>,
    R: Results,
{
    type View = (C, R::View);
    type Iterator = iter::Zip<I, R::Iterator>;

    fn into_iterator(self) -> Self::Iterator {
        self.0.zip(self.1.into_iterator())
    }
}
