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

use crate::{
    archetype,
    entity,
    registry::Registry,
};
use alloc::{
    collections::VecDeque,
    vec::Vec,
};
use by_address::ByThinAddress;
use core::{
    fmt,
    fmt::Debug,
};
use fnv::FnvBuildHasher;
use hashbrown::HashMap;

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

    pub(crate) fn allocate(&mut self, location: Location<R>) -> entity::Identifier {
        let (index, generation) = if let Some(index) = self.free.pop_front() {
            let slot =
                // SAFETY: `self.free` is guaranteed to contain valid indices within the bounds of
                // `self.slots`.
                unsafe {self.slots.get_unchecked_mut(index)};
            // SAFETY: `self.free` is guaranteed to contain indices for slots that are not active.
            unsafe { slot.activate_unchecked(location) };
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
    pub(crate) fn allocate_batch(
        &mut self,
        mut locations: Locations<R>,
    ) -> Vec<entity::Identifier> {
        let mut identifiers = Vec::with_capacity(locations.len());

        // First activate slots that are already allocated.
        while let Some(index) = self.free.pop_front() {
            if locations.is_empty() {
                break;
            }
            let slot =
                // SAFETY: indices within `self.free` are guaranteed to be within the bounds of
                // `self.slots`.
                unsafe { self.slots.get_unchecked_mut(index) };
            // SAFETY: `self.free` is guaranteed to contain indices for slots that are not active.
            // Also, `locations` is already checked above to be nonempty.
            unsafe { slot.activate_unchecked(locations.next().unwrap_unchecked()) };
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

    /// Free the entity allocation identified by `identifier`, skipping checks for whether the
    /// allocation exists.
    ///
    /// # Safety
    /// `identifier` must be for a valid, currently allocated entity.
    pub(crate) unsafe fn free_unchecked(&mut self, identifier: entity::Identifier) {
        let slot =
            // SAFETY: `identifier` is guaranteed by the safety contract of this method to identify
            // a valid entity. Therefore, its `index` will correspond to a valid value within
            // `self.slots`.
            unsafe {self.slots.get_unchecked_mut(identifier.index)};
        slot.deactivate();
        self.free.push_back(identifier.index);
    }

    /// Update the location of the entity identified by `identifier`, skipping checks for whether
    /// the allocation exists.
    ///
    /// # Safety
    /// `identifier` must be for a valid, currently allocated entity.
    pub(crate) unsafe fn modify_location_unchecked(
        &mut self,
        identifier: entity::Identifier,
        location: Location<R>,
    ) {
        // SAFETY: `identifier` is guaranteed by the safety contract of this method to identify a
        // valid entity. Therefore, its `index` will correspond to a valid value within
        // `self.slots`.
        unsafe { self.slots.get_unchecked_mut(identifier.index) }.location = Some(location);
    }

    /// Update the location's index of the entity identified by `identifier`, skipping checks for
    /// whether the allocation exists.
    ///
    /// This should be used when an entity's location within an archetype table has changed.
    /// Calling this method ensures the allocator's mapping of where entities currently are stays
    /// up to date.
    ///
    /// # Safety
    /// `identifier` must be for a valid, currently allocated entity.
    pub(crate) unsafe fn modify_location_index_unchecked(
        &mut self,
        identifier: entity::Identifier,
        index: usize,
    ) {
        // SAFETY: `identifier` is guaranteed by the safety contract of this method to identify a
        // valid active entity. Therefore, its `index` will correspond to a valid active value
        // within `self.slots`.
        unsafe {
            self.slots
                .get_unchecked_mut(identifier.index)
                .location
                .as_mut()
                .unwrap_unchecked()
        }
        .index = index;
    }

    /// Decrease the allocated capacity to the smallest amount required for the stored data.
    ///
    /// This may not decrease to the most optimal value, as the shrinking is dependent on the
    /// allocator.
    ///
    /// Note that this only affects the list of currently free indexes. Slots are never removed, so
    /// there is no need to shrink them.
    pub(crate) fn shrink_to_fit(&mut self) {
        self.free.shrink_to_fit();
    }

    /// Clone the entity allocator, using `identifier_map` to replace old archetype identifiers
    /// with new ones.
    ///
    /// # Safety
    /// `identifier_map` must contain entries for every archetype referenced in this entity
    /// allocator.
    pub(crate) unsafe fn clone(
        &self,
        identifier_map: &HashMap<ByThinAddress<&[u8]>, archetype::IdentifierRef<R>, FnvBuildHasher>,
    ) -> Self {
        Self {
            slots: self
                .slots
                .iter()
                .map(|slot|
                // SAFETY: `identifier_map` contains an entry for the archetype referenced in
                // `slot`, if there is one.
                unsafe {slot.clone_with_new_identifier(identifier_map)})
                .collect(),
            free: self.free.clone(),
        }
    }

    /// Clone from another entity allocator into this one, using `identifier_map` to replace old
    /// archetype identifiers with new ones.
    ///
    /// This reuses the existing allocations.
    ///
    /// # Safety
    /// `identifier_map` must contain entries for every archetype referenced in this entity
    /// allocator.
    pub(crate) unsafe fn clone_from(
        &mut self,
        source: &Self,
        identifier_map: &HashMap<ByThinAddress<&[u8]>, archetype::IdentifierRef<R>, FnvBuildHasher>,
    ) {
        self.slots.clear();
        self.slots.extend(source.slots.iter().map(|slot|
            // SAFETY: `identifier_map` contains an entry for the archetype referenced in
            // `slot`, if there is one.
            unsafe {slot.clone_with_new_identifier(identifier_map)}));

        self.free.clone_from(&source.free);
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
