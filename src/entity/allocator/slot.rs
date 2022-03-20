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
    pub(super) fn new(location: Location<R>) -> Self {
        Self {
            generation: 0,
            location: Some(location),
        }
    }

    /// Activate this slot without checking whether it is already activated.
    ///
    /// If the current slot is already active, this will lead to invalidation of
    /// `entity::Identifier`s prematurely and make it impossible to remove entities from the
    /// `World`. Additionally, it would leave invalid identifiers within archetype tables, causing
    /// some improper assumptions. Therefore, this method is considered `unsafe` and must be called
    /// with care.
    ///
    /// # Safety
    /// A `Slot` this method is called on must not already be active.
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
