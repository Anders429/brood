use crate::{entity::allocator::Location, registry::Registry};
use core::{fmt, fmt::Debug};

/// An entry for a possibly allocated entity.
///
/// If this slot has a stored location, then an entity is allocated at that location. If the
/// location is `None`, then the slot is free and can be used to store a new entity. When the slot
/// has a stored location, it is called "active".
///
/// Slots are reused. To differentiate between different allocations that have shared the same
/// slot, a unique generation is used. Therefore, a unique entity is determined both by its slot
/// index and its slot's generation.
pub(crate) struct Slot<R>
where
    R: Registry,
{
    /// The currently stored entity's generation.
    pub(crate) generation: u64,
    /// The location of the entity, if one is currently allocated.
    ///
    /// A `None` value indicates no entity is allocated in this slot.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{archetype::Identifier, registry};
    use alloc::vec;
    use claims::{assert_none, assert_some_eq};

    macro_rules! create_components {
        ($( $variants:ident ),*) => {
            $(
                struct $variants(f32);
            )*
        };
    }

    create_components!(
        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
    );

    type Registry =
        registry!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

    #[test]
    fn new() {
        let identifier = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let location = Location::new(unsafe { identifier.as_ref() }, 42);
        let slot = Slot::new(location);

        assert_eq!(slot.generation, 0);
        assert_some_eq!(slot.location, location);
        assert!(slot.is_active());
    }

    #[test]
    fn deactivate() {
        let identifier = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let location = Location::new(unsafe { identifier.as_ref() }, 42);
        let mut slot = Slot::new(location);

        slot.deactivate();

        assert_eq!(slot.generation, 0);
        assert_none!(slot.location);
        assert!(!slot.is_active());
    }

    #[test]
    fn activate_unchecked() {
        let identifier = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let location = Location::new(unsafe { identifier.as_ref() }, 42);
        let mut slot = Slot::new(location);

        slot.deactivate();
        let new_identifier = unsafe { Identifier::<Registry>::new(vec![3, 2, 1, 0]) };
        let new_location = Location::new(unsafe { new_identifier.as_ref() }, 42);
        unsafe {
            slot.activate_unchecked(new_location);
        }

        assert_eq!(slot.generation, 1);
        assert_some_eq!(slot.location, new_location);
        assert!(slot.is_active());
    }
}
