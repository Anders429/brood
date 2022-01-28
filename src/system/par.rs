use crate::{
    query::{filter::Filter, result, view::ParViews},
    registry::Registry,
    world::World,
};

#[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
pub trait ParSystem<'a> {
    type Filter: Filter;
    type Views: ParViews<'a>;

    fn run<R>(&mut self, query_results: result::ParIter<'a, R, Self::Filter, Self::Views>)
    where
        R: Registry + 'a;

    #[inline]
    #[allow(unused_variables)]
    fn world_post_processing<R>(&mut self, world: &mut World<R>)
    where
        R: Registry,
    {
    }
}
