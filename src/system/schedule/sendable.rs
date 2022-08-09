use crate::{registry::Registry, world::World};

pub struct SendableWorld<R>(*mut World<R>)
where
    R: Registry;

impl<R> SendableWorld<R>
where
    R: Registry,
{
    /// # Safety
    /// The `world` pointer passed here must be exclusively.
    pub(crate) unsafe fn new(world: *mut World<R>) -> Self {
        Self(world)
    }

    /// # Safety
    /// The pointer returned here must only be used for access to components that follow Rust's
    /// borrowing rules.
    pub(crate) unsafe fn get(self) -> *mut World<R> {
        self.0
    }
}

impl<R> Clone for SendableWorld<R>
where
    R: Registry,
{
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<R> Copy for SendableWorld<R> where R: Registry {}

// SAFETY: This type can be safely sent between threads as long as the safety contracts of its
// methods are upheld, because the data accessed will be accessed uniquely.
unsafe impl<R> Send for SendableWorld<R> where R: Registry {}

// SAFETY: This type can be safely shared between threads as long as the safety contracts of its
// methods are upheld, because the data accessed will be accessed uniquely, including mutable
// reference access.
unsafe impl<R> Sync for SendableWorld<R> where R: Registry {}
