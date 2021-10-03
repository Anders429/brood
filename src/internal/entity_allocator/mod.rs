// #[cfg(feature = "serde")]
// mod impl_serde;

use crate::entity::EntityIdentifier;
use alloc::{collections::VecDeque, vec::Vec};

#[derive(Clone, Debug)]
struct Slot {
    generation: u64,
    active: bool,
    key: *const u8,
}

impl Slot {
    unsafe fn new(key: *const u8) -> Self {
        Self {
            generation: 0,
            active: true,
            key,
        }
    }

    unsafe fn activate_unchecked(&mut self, key: *const u8) {
        self.generation = self.generation.wrapping_add(1);
        self.active = true;
        self.key = key;
    }
}

#[derive(Debug)]
pub(crate) struct EntityAllocator {
    slots: Vec<Slot>,
    free: VecDeque<usize>,
}

impl EntityAllocator {
    pub(crate) fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: VecDeque::new(),
        }
    }

    pub(crate) unsafe fn allocate(&mut self, key: *const u8) -> EntityIdentifier {
        let (index, generation) = if let Some(index) = self.free.pop_front() {
            let slot = self.slots.get_unchecked_mut(index);
            slot.activate_unchecked(key);
            (index, slot.generation)
        } else {
            let index = self.slots.len();
            self.slots.push(Slot::new(key));
            // Generation is always 0 for a new slot.
            (index, 0)
        };

        EntityIdentifier::new(index, generation)
    }

    pub(crate) unsafe fn allocate_batch(
        &mut self,
        key: *const u8,
        mut batch_size: usize,
    ) -> impl Iterator<Item = EntityIdentifier> {
        let mut identifiers = Vec::with_capacity(batch_size);

        // First activate slots that are already allocated.
        while let Some(index) = self.free.pop_front() {
            if batch_size == 0 {
                break;
            }
            let slot = self.slots.get_unchecked_mut(index);
            slot.activate_unchecked(key);
            identifiers.push(EntityIdentifier::new(index, slot.generation));
            batch_size -= 1;
        }

        // Now allocate the remaining slots.
        let slots_len = self.slots.len();
        self.slots.resize(slots_len + batch_size, Slot::new(key));
        identifiers
            .extend((0..batch_size).map(|index| EntityIdentifier::new(slots_len + index, 0)));

        identifiers.into_iter()
    }
}
