#[cfg(feature = "rayon")]
use crate::system::ParSystem;
use crate::{
    query::{filter, filter::Filter, result, view},
    registry::Registry,
    system::System,
    world::World,
};
use core::hint::unreachable_unchecked;

/// A null system.
///
/// As this is an empty `enum`, it can never be instantiated. Its main use is as a generic argument
/// for [`Schedule`]s for the unused generic parameters of [`Stage`]s.
///
/// [`Schedule`]: crate::system::Schedule
/// [`Stage`]: crate::system::schedule::stage::Stage
pub enum Null {}

impl<'a> System<'a> for Null {
    type Filter = filter::None;
    type Views = view::Null;

    fn run<R, FI, VI>(
        &mut self,
        _query_results: result::Iter<'a, R, Self::Filter, FI, Self::Views, VI>,
    ) where
        R: Registry + 'a,
        Self::Filter: Filter<R, FI>,
        Self::Views: Filter<R, VI>,
    {
        // SAFETY: This type can never be instantiated. Therefore, this method can never be called.
        unsafe { unreachable_unchecked() }
    }

    fn world_post_processing<R>(&mut self, _world: &mut World<R>)
    where
        R: Registry,
    {
        // SAFETY: This type can never be instantiated. Therefore, this method can never be called.
        unsafe { unreachable_unchecked() }
    }
}

#[cfg(feature = "rayon")]
impl<'a> ParSystem<'a> for Null {
    type Filter = filter::None;
    type Views = view::Null;

    fn run<R, FI, VI>(
        &mut self,
        _query_results: result::ParIter<'a, R, Self::Filter, FI, Self::Views, VI>,
    ) where
        R: Registry + 'a,
        Self::Filter: Filter<R, FI>,
        Self::Views: Filter<R, VI>,
    {
        // SAFETY: This type can never be instantiated. Therefore, this method can never be called.
        unsafe { unreachable_unchecked() }
    }

    fn world_post_processing<R>(&mut self, _world: &mut World<R>)
    where
        R: Registry,
    {
        // SAFETY: This type can never be instantiated. Therefore, this method can never be called.
        unsafe { unreachable_unchecked() }
    }
}
