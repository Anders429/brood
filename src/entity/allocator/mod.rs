#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;
mod location;
mod locations;
mod slot;

#[cfg(feature = "serde")]
pub(crate) use impl_serde::DeserializeAllocator;
pub(crate) use location::Location;
pub(crate) use locations::Locations;
pub(crate) use slot::Slot;

use crate::{entity, registry::Registry};
use alloc::{collections::VecDeque, vec::Vec};
use core::{fmt, fmt::Debug};

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
    pub(crate) unsafe fn allocate_batch(
        &mut self,
        mut locations: Locations<R>,
    ) -> Vec<entity::Identifier> {
        let mut identifiers = Vec::with_capacity(locations.len());

        // First activate slots that are already allocated.
        while let Some(index) = self.free.pop_front() {
            if locations.is_empty() {
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

    pub(crate) fn is_active(&self, identifier: entity::Identifier) -> bool {
        if let Some(slot) = self.slots.get(identifier.index) {
            if slot.is_active() && slot.generation == identifier.generation {
                return true;
            }
        }
        false
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
        self.slots
            .get_unchecked_mut(identifier.index)
            .location
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
