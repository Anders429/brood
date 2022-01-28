mod seal;

use crate::{
    component::Component,
    entity,
    query::{filter::Filter, view::Null},
};
use seal::{ParViewSeal, ParViewsSeal};

#[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
pub trait ParView<'a>: Filter + ParViewSeal<'a> {}

impl<'a, C> ParView<'a> for &C where C: Component + Sync {}

impl<'a, C> ParView<'a> for &mut C where C: Component + Send {}

impl<'a, C> ParView<'a> for Option<&C> where C: Component + Sync {}

impl<'a, C> ParView<'a> for Option<&mut C> where C: Component + Send {}

impl<'a> ParView<'a> for entity::Identifier {}

#[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
pub trait ParViews<'a>: Filter + ParViewsSeal<'a> {}

impl<'a> ParViews<'a> for Null {}

impl<'a, V, W> ParViews<'a> for (V, W)
where
    V: ParView<'a>,
    W: ParViews<'a>,
{
}
