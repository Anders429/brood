use crate::{
    component::Component,
    internal::query::{ViewSeal, ViewsSeal},
    query::{Filter, NullResult},
};
use alloc::vec;
use core::{iter, marker::PhantomData, slice};

pub trait View<'a>: Filter + ViewSeal<'a> {}

pub struct Read<C>
where
    C: Component,
{
    component: PhantomData<C>,
}

impl<'a, C> View<'a> for Read<C> where C: Component {}

pub struct Write<C>
where
    C: Component,
{
    component: PhantomData<C>,
}

impl<'a, C> View<'a> for Write<C> where C: Component {}

pub struct NullViews;

pub trait Views<'a>: Filter + ViewsSeal {
    type Results;
}

impl<'a> Views<'a> for NullViews {
    type Results = iter::Repeat<NullResult>;
}

impl<'a, V, W> Views<'a> for (V, W)
where
    V: View<'a>,
    W: Views<'a>,
{
    type Results = iter::Zip<iter::Flatten<vec::IntoIter<V::Result>>, W::Results>;
}
