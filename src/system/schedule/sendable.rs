use crate::{registry::Registry, world::World};

pub struct SendableWorld<'a, R>(pub(crate) &'a World<R>)
where
    R: Registry;

impl<R> Clone for SendableWorld<'_, R>
where
    R: Registry,
{
    fn clone(&self) -> Self {
        Self(<&World<R>>::clone(&self.0))
    }
}

impl<R> Copy for SendableWorld<'_, R> where R: Registry {}

unsafe impl<R> Send for SendableWorld<'_, R> where R: Registry {}

unsafe impl<R> Sync for SendableWorld<'_, R> where R: Registry {}
