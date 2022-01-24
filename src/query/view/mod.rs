#[cfg(feature = "parallel")]
mod par;
mod seal;

#[cfg(feature = "parallel")]
pub use par::{ParView, ParViews};

use crate::{component::Component, entity, hlist::define_null, query::filter::Filter};
use seal::{ViewSeal, ViewsSeal};

pub trait View<'a>: Filter + ViewSeal<'a> {}

impl<'a, C> View<'a> for &C where C: Component {}

impl<'a, C> View<'a> for &mut C where C: Component {}

impl<'a, C> View<'a> for Option<&C> where C: Component {}

impl<'a, C> View<'a> for Option<&mut C> where C: Component {}

impl<'a> View<'a> for entity::Identifier {}

define_null!();

pub trait Views<'a>: Filter + ViewsSeal<'a> {}

impl<'a> Views<'a> for Null {}

impl<'a, V, W> Views<'a> for (V, W)
where
    V: View<'a>,
    W: Views<'a>,
{
}

#[macro_export]
macro_rules! views {
    ($view:ty $(,$views:ty)* $(,)?) => {
        ($view, $crate::views!($($views,)*))
    };
    () => {
        $crate::query::view::Null
    };
}
