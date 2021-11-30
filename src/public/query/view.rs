use crate::{
    component::Component,
    internal::query::{ViewSeal, ViewsSeal},
    query::Filter,
};
use core::marker::PhantomData;

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

pub trait Views<'a>: Filter + ViewsSeal<'a> {}

impl<'a> Views<'a> for NullViews {}

impl<'a, V, W> Views<'a> for (V, W)
where
    V: View<'a>,
    W: Views<'a>,
{
}
