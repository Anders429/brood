use crate::{registry::Registry, world::World};

pub struct SendableWorld<R>(pub(crate) *mut World<R>)
where
    R: Registry;

impl<R> Clone for SendableWorld<R>
where
    R: Registry,
{
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<R> Copy for SendableWorld<R> where R: Registry {}

unsafe impl<R> Send for SendableWorld<R> where R: Registry {}

unsafe impl<R> Sync for SendableWorld<R> where R: Registry {}
