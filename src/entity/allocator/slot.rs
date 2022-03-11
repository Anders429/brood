use crate::{entity::allocator::Location, registry::Registry};
use core::{fmt, fmt::Debug};

pub(crate) struct Slot<R>
where
    R: Registry,
{
    pub(crate) generation: u64,
    pub(crate) location: Option<Location<R>>,
}

impl<R> Slot<R>
where
    R: Registry,
{
    // TODO: Should this really be considered unsafe?
    pub(super) unsafe fn new(location: Location<R>) -> Self {
        Self {
            generation: 0,
            location: Some(location),
        }
    }

    // TODO: Should this really be considered unsafe?
    pub(super) unsafe fn activate_unchecked(&mut self, location: Location<R>) {
        self.generation = self.generation.wrapping_add(1);
        self.location = Some(location);
    }

    pub(super) fn deactivate(&mut self) {
        self.location = None;
    }

    pub(super) fn is_active(&self) -> bool {
        self.location.is_some()
    }
}

impl<R> Clone for Slot<R>
where
    R: Registry,
{
    fn clone(&self) -> Self {
        Self {
            generation: self.generation,
            location: self.location,
        }
    }
}

impl<R> Debug for Slot<R>
where
    R: Registry,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Slot")
            .field("generation", &self.generation)
            .field("location", &self.location)
            .finish()
    }
}

impl<R> PartialEq for Slot<R>
where
    R: Registry,
{
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation && self.location == other.location
    }
}
