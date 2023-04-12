mod repeat;

pub(crate) use repeat::RepeatNone;

use crate::{
    component::Component,
    entity,
    query::{
        result::ParResults,
        view::{
            Null,
            View,
            Views,
        },
    },
};
use rayon::{
    iter,
    iter::{
        Either,
        IndexedParallelIterator,
    },
    slice,
};

pub trait ParViewSeal<'a>: View<'a> {
    type ParResult: IndexedParallelIterator<Item = Self>;
}

impl<'a, C> ParViewSeal<'a> for &'a C
where
    C: Component + Sync,
{
    type ParResult = slice::Iter<'a, C>;
}

impl<'a, C> ParViewSeal<'a> for &'a mut C
where
    C: Component + Send,
{
    type ParResult = slice::IterMut<'a, C>;
}

impl<'a, C> ParViewSeal<'a> for Option<&'a C>
where
    C: Component + Sync,
{
    type ParResult = Either<
        iter::RepeatN<Option<&'a C>>,
        iter::Map<slice::Iter<'a, C>, fn(&'a C) -> Option<&'a C>>,
    >;
}

impl<'a, C> ParViewSeal<'a> for Option<&'a mut C>
where
    C: Component + Send,
{
    type ParResult = Either<
        RepeatNone<&'a mut C>,
        iter::Map<slice::IterMut<'a, C>, fn(&'a mut C) -> Option<&'a mut C>>,
    >;
}

impl<'a> ParViewSeal<'a> for entity::Identifier {
    type ParResult = iter::Cloned<slice::Iter<'a, Self>>;
}

pub trait ParViewsSeal<'a>: Views<'a> {
    type ParResults: ParResults<View = Self>;
}

impl<'a> ParViewsSeal<'a> for Null {
    type ParResults = iter::RepeatN<Null>;
}

impl<'a, V, W> ParViewsSeal<'a> for (V, W)
where
    V: ParViewSeal<'a> + Send,
    W: ParViewsSeal<'a>,
{
    type ParResults = (V::ParResult, W::ParResults);
}
