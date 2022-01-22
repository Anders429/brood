#[cfg(feature = "parallel")]
pub mod schedule;

mod null;
#[cfg(feature = "parallel")]
mod par;

pub use null::Null;
#[cfg(feature = "parallel")]
pub use par::ParSystem;
#[cfg(feature = "parallel")]
pub use schedule::Schedule;

use crate::{
    query::{filter::Filter, result, view::Views},
    registry::Registry,
    world::World,
};

pub trait System<'a> {
    type Filter: Filter;
    type Views: Views<'a>;

    fn run<R>(&mut self, query_results: result::Iter<'a, R, Self::Filter, Self::Views>)
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
