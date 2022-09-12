use crate::{
    component::Component,
    entity,
    query::{claim::Claim, result::Results, view::Null},
};
use core::{iter, slice};
use either::Either;

pub trait ViewSeal<'a>: Claim {
    type Result: Iterator<Item = Self>;
}

impl<'a, C> ViewSeal<'a> for &'a C
where
    C: Component,
{
    type Result = slice::Iter<'a, C>;
}

impl<'a, C> ViewSeal<'a> for &'a mut C
where
    C: Component,
{
    type Result = slice::IterMut<'a, C>;
}

impl<'a, C> ViewSeal<'a> for Option<&'a C>
where
    C: Component,
{
    type Result = Either<
        iter::Take<iter::Repeat<Option<&'a C>>>,
        iter::Map<slice::Iter<'a, C>, fn(&'a C) -> Option<&'a C>>,
    >;
}

impl<'a, C> ViewSeal<'a> for Option<&'a mut C>
where
    C: Component,
{
    type Result = Either<
        iter::Take<iter::RepeatWith<fn() -> Option<&'a mut C>>>,
        iter::Map<slice::IterMut<'a, C>, fn(&'a mut C) -> Option<&'a mut C>>,
    >;
}

impl<'a> ViewSeal<'a> for entity::Identifier {
    type Result = iter::Copied<slice::Iter<'a, Self>>;
}

pub trait ViewsSeal<'a>: Claim {
    type Results: Results<View = Self>;
}

impl<'a> ViewsSeal<'a> for Null {
    type Results = iter::Repeat<Null>;
}

impl<'a, V, W> ViewsSeal<'a> for (V, W)
where
    V: ViewSeal<'a>,
    W: ViewsSeal<'a>,
{
    type Results = (V::Result, W::Results);
}
