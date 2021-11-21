use crate::entity::EntityIdentifier;
use alloc::{collections::VecDeque, vec::Vec};
use core::{iter::ExactSizeIterator, ptr};

#[derive(Copy, Clone, Debug)]
pub(crate) struct Location {
    pub(crate) key: ptr::NonNull<u8>,
    pub(crate) index: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct Slot {
    pub(crate) generation: u64,
    pub(crate) location: Option<Location>,
}

impl Slot {
    unsafe fn new(location: Location) -> Self {
        Self {
            generation: 0,
            location: Some(location),
        }
    }

    unsafe fn activate_unchecked(&mut self, location: Location) {
        self.generation = self.generation.wrapping_add(1);
        self.location = Some(location);
    }
}

#[derive(Debug)]
pub struct EntityAllocator {
    pub(crate) slots: Vec<Slot>,
    pub(crate) free: VecDeque<usize>,
}

impl EntityAllocator {
    pub(crate) fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: VecDeque::new(),
        }
    }

    pub(crate) unsafe fn allocate(&mut self, location: Location) -> EntityIdentifier {
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

        EntityIdentifier::new(index, generation)
    }

    pub(crate) unsafe fn allocate_batch<L>(
        &mut self,
        mut locations: L,
    ) -> impl Iterator<Item = EntityIdentifier>
    where
        L: Iterator<Item = Location> + ExactSizeIterator,
    {
        let mut identifiers = Vec::with_capacity(locations.len());

        // First activate slots that are already allocated.
        while let Some(index) = self.free.pop_front() {
            if locations.len() == 0 {
                break;
            }
            let slot = self.slots.get_unchecked_mut(index);
            slot.activate_unchecked(locations.next().unwrap_unchecked());
            identifiers.push(EntityIdentifier::new(index, slot.generation));
        }

        // Now allocate the remaining slots.
        let remaining_locations = locations.len();
        let slots_len = self.slots.len();
        self.slots
            .extend(locations.map(|location| Slot::new(location)));
        identifiers.extend(
            (0..remaining_locations).map(|index| EntityIdentifier::new(slots_len + index, 0)),
        );

        identifiers.into_iter()
    }

    pub(crate) fn get(&self, identifier: EntityIdentifier) -> Option<Location> {
        let slot = self.slots.get(identifier.index)?;
        if slot.generation == identifier.generation {
            slot.location
        } else {
            None
        }
    }
}
