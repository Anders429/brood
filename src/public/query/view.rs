use crate::{
    component::Component,
    entity::EntityIdentifier,
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

impl<'a> View<'a> for EntityIdentifier {}

pub struct NullViews;

pub trait Views<'a>: Filter + ViewsSeal<'a> {}

impl<'a> Views<'a> for NullViews {}

impl<'a, V, W> Views<'a> for (V, W)
where
    V: View<'a>,
    W: Views<'a>,
{
}

#[macro_export]
macro_rules! views {
    ($view:ty $(,$views:ty)* $(,)?) => {
        ($view, views!($($views,)*))
    };
    () => {
        $crate::query::NullViews
    };
}
