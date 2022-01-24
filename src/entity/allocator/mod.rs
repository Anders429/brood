#[cfg(feature = "serde")]
mod impl_serde;

#[cfg(feature = "serde")]
pub(crate) use impl_serde::DeserializeAllocator;

use crate::{archetype, entity, registry::Registry};
use alloc::{collections::VecDeque, vec::Vec};
use core::{fmt, fmt::Debug, iter::ExactSizeIterator};

pub(crate) struct Location<R>
where
    R: Registry,
{
    pub(crate) identifier: archetype::IdentifierRef<R>,
    pub(crate) index: usize,
}

impl<R> Location<R>
where
    R: Registry,
{
    pub(crate) fn new(identifier: archetype::IdentifierRef<R>, index: usize) -> Self {
        Self { identifier, index }
    }
}

impl<R> Clone for Location<R>
where
    R: Registry,
{
    fn clone(&self) -> Self {
        Self {
            identifier: self.identifier,
            index: self.index,
        }
    }
}

impl<R> Copy for Location<R> where R: Registry {}

impl<R> Debug for Location<R>
where
    R: Registry,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Location")
            .field("identifier", &self.identifier)
            .field("index", &self.index)
            .finish()
    }
}

impl<R> PartialEq for Location<R>
where
    R: Registry,
{
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier && self.index == other.index
    }
}

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
    unsafe fn new(location: Location<R>) -> Self {
        Self {
            generation: 0,
            location: Some(location),
        }
    }

    // TODO: Should this really be considered unsafe?
    unsafe fn activate_unchecked(&mut self, location: Location<R>) {
        self.generation = self.generation.wrapping_add(1);
        self.location = Some(location);
    }

    fn deactivate(&mut self) {
        self.location = None;
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

pub struct Allocator<R>
where
    R: Registry,
{
    pub(crate) slots: Vec<Slot<R>>,
    pub(crate) free: VecDeque<usize>,
}

impl<R> Allocator<R>
where
    R: Registry,
{
    pub(crate) fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: VecDeque::new(),
        }
    }

    pub(crate) unsafe fn allocate(&mut self, location: Location<R>) -> entity::Identifier {
        let (index, generation) = if let Some(index) = self.free.pop_front() {
            let slot = self.slots.get_unchecked_mut(index);
            slot.activate_unchecked(location);
            (index, slot.generation)
        } else {
            let index = self.slots.len();
            self.slots.push(Slot::new(location));
            // Generation is always 0 for a new slot.
            (index, 0)
        };

        entity::Identifier::new(index, generation)
    }

    #[inline]
    pub(crate) unsafe fn allocate_batch<L>(&mut self, mut locations: L) -> Vec<entity::Identifier>
    where
        L: Iterator<Item = Location<R>> + ExactSizeIterator,
    {
        let mut identifiers = Vec::with_capacity(locations.len());

        // First activate slots that are already allocated.
        while let Some(index) = self.free.pop_front() {
            if locations.len() == 0 {
                break;
            }
            let slot = self.slots.get_unchecked_mut(index);
            slot.activate_unchecked(locations.next().unwrap_unchecked());
            identifiers.push(entity::Identifier::new(index, slot.generation));
        }

        // Now allocate the remaining slots.
        let remaining_locations = locations.len();
        let slots_len = self.slots.len();
        self.slots
            .extend(locations.map(|location| Slot::new(location)));
        identifiers.extend(
            (0..remaining_locations).map(|index| entity::Identifier::new(slots_len + index, 0)),
        );

        identifiers
    }

    pub(crate) fn get(&self, identifier: entity::Identifier) -> Option<Location<R>> {
        let slot = self.slots.get(identifier.index)?;
        if slot.generation == identifier.generation {
            slot.location
        } else {
            None
        }
    }

    pub(crate) unsafe fn free_unchecked(&mut self, identifier: entity::Identifier) {
        let slot = self.slots.get_unchecked_mut(identifier.index);
        slot.deactivate();
        self.free.push_back(identifier.index);
    }

    pub(crate) unsafe fn modify_location_unchecked(
        &mut self,
        identifier: entity::Identifier,
        location: Location<R>,
    ) {
        self.slots.get_unchecked_mut(identifier.index).location = Some(location);
    }

    pub(crate) unsafe fn modify_location_index_unchecked(
        &mut self,
        identifier: entity::Identifier,
        index: usize,
    ) {
        (&mut self.slots.get_unchecked_mut(identifier.index).location)
            .as_mut()
            .unwrap_unchecked()
            .index = index;
    }
}

impl<R> Debug for Allocator<R>
where
    R: Registry,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Allocator")
            .field("slots", &self.slots)
            .field("free", &self.free)
            .finish()
    }
}

impl<R> PartialEq for Allocator<R>
where
    R: Registry,
{
    fn eq(&self, other: &Self) -> bool {
        self.slots == other.slots && self.free == other.free
    }
}