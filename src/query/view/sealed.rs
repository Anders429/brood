use crate::{
    component::Component,
    entity,
    query::{
        filter,
        result::Results,
        view::Null,
    },
};
use core::{
    iter,
    mem::MaybeUninit,
    slice,
};
use either::Either;

pub trait ViewSealed<'a> {
    type Result: Iterator<Item = Self>;
    type Index;
    type MaybeUninit;
    type EntryFilter;
}

impl<'a, C> ViewSealed<'a> for &'a C
where
    C: Component,
{
    type Result = slice::Iter<'a, C>;
    type Index = usize;
    type MaybeUninit = MaybeUninit<Self>;
    type EntryFilter = filter::Has<C>;
}

impl<'a, C> ViewSealed<'a> for &'a mut C
where
    C: Component,
{
    type Result = slice::IterMut<'a, C>;
    type Index = usize;
    type MaybeUninit = MaybeUninit<Self>;
    type EntryFilter = filter::Has<C>;
}

impl<'a, C> ViewSealed<'a> for Option<&'a C>
where
    C: Component,
{
    type Result = Either<
        iter::Take<iter::Repeat<Option<&'a C>>>,
        iter::Map<slice::Iter<'a, C>, fn(&'a C) -> Option<&'a C>>,
    >;
    type Index = usize;
    type MaybeUninit = Self;
    type EntryFilter = filter::Has<C>;
}

impl<'a, C> ViewSealed<'a> for Option<&'a mut C>
where
    C: Component,
{
    type Result = Either<
        iter::Take<iter::RepeatWith<fn() -> Option<&'a mut C>>>,
        iter::Map<slice::IterMut<'a, C>, fn(&'a mut C) -> Option<&'a mut C>>,
    >;
    type Index = usize;
    type MaybeUninit = Self;
    type EntryFilter = filter::Has<C>;
}

impl<'a> ViewSealed<'a> for entity::Identifier {
    type Result = iter::Copied<slice::Iter<'a, Self>>;
    type Index = Null;
    type MaybeUninit = Self;
    type EntryFilter = filter::Not<filter::None>;
}

pub trait ViewsSealed<'a> {
    type Results: Results<View = Self>;
    type Indices;
    type MaybeUninit;
    type EntryFilter;
}

impl<'a> ViewsSealed<'a> for Null {
    type Results = iter::Take<iter::Repeat<Null>>;
    type Indices = Null;
    type MaybeUninit = Null;
    type EntryFilter = filter::Not<filter::None>;
}

impl<'a, V, W> ViewsSealed<'a> for (V, W)
where
    V: ViewSealed<'a>,
    W: ViewsSealed<'a>,
{
    type Results = (V::Result, W::Results);
    type Indices = (V::Index, W::Indices);
    type MaybeUninit = (V::MaybeUninit, W::MaybeUninit);
    type EntryFilter = filter::Or<W::EntryFilter, V::EntryFilter>;
}
