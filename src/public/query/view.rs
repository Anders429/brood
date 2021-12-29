use crate::{
    component::Component,
    entity::EntityIdentifier,
    internal::query::{ViewSeal, ViewsSeal},
    query::Filter,
};

pub trait View<'a>: Filter + ViewSeal<'a> {}

impl<'a, C> View<'a> for &C where C: Component {}

impl<'a, C> View<'a> for &mut C where C: Component {}

impl<'a, C> View<'a> for Option<&C> where C: Component {}

impl<'a, C> View<'a> for Option<&mut C> where C: Component {}

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

pub struct WrapSome<I> where I: Iterator {
    pub(crate) iter: I,
}

impl<I> Iterator for WrapSome<I> where I: Iterator {
    type Item = Option<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next())
    }
}
