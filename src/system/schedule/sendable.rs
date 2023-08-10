use crate::{
    registry::Registry,
    world::World,
};

pub struct SendableWorld<R, Resources>(*mut World<R, Resources>)
where
    R: Registry;

impl<R, Resources> SendableWorld<R, Resources>
where
    R: Registry,
{
    /// # Safety
    /// The `world` pointer passed here must be exclusively.
    pub(crate) unsafe fn new(world: *mut World<R, Resources>) -> Self {
        Self(world)
    }

    /// # Safety
    /// The pointer returned here must only be used for access to components that follow Rust's
    /// borrowing rules.
    pub(crate) unsafe fn get(self) -> *mut World<R, Resources> {
        self.0
    }
}

impl<R, Resources> Clone for SendableWorld<R, Resources>
where
    R: Registry,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<R, Resources> Copy for SendableWorld<R, Resources> where R: Registry {}

// SAFETY: This type can be safely sent between threads as long as the safety contracts of its
// methods are upheld, because the data accessed will be accessed uniquely.
unsafe impl<R, Resources> Send for SendableWorld<R, Resources> where R: Registry {}

// SAFETY: This type can be safely shared between threads as long as the safety contracts of its
// methods are upheld, because the data accessed will be accessed uniquely, including mutable
// reference access.
unsafe impl<R, Resources> Sync for SendableWorld<R, Resources> where R: Registry {}
