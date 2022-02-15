#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;

#[cfg(feature = "serde")]
pub(crate) use impl_serde::DeserializeAllocator;

use crate::{archetype, entity, registry::Registry};
use alloc::{collections::VecDeque, vec::Vec};
use core::{fmt, fmt::Debug, ops::Range};

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

/// A batch of [`Location`]s, all sharing the same [`archetype::Identifier`].
///
/// These locations are meant to be iterated over, yielding the appropriate `Location`s defined.
///
/// [`archetype::Identifier`]: crate::archetype::Identifier
/// [`Location`]: crate::entity::allocator::Location
pub(crate) struct Locations<R>
where
    R: Registry,
{
    /// The remaining indices to be iterated.
    indices: Range<usize>,
    /// The archetype identifier of all contained [`Location`]s.
    ///
    /// [`Location`]: crate::entity::allocator::Location
    identifier: archetype::IdentifierRef<R>,
}

impl<R> Locations<R>
where
    R: Registry,
{
    /// Create a new batch of [`Location`]s.
    ///
    /// This method takes a range of indices, representing the indices of entities within an
    /// [`Archetype`], and a single [`archetype::Identifier`] to specify which `Archetype` the
    /// entities belong to.
    ///
    /// Note that a single `Locations` container cannot be used to represent locations from
    /// multiple `Archetype`s.
    ///
    /// Note that it is an error if `end` is less than `start` within `indices` (although not
    /// technically `unsafe`).
    ///
    /// [`Archetype`]: crate::archetype::Archetype
    /// [`archetype::Identifier`]: crate::archetype::Identifier
    /// [`Location`]: crate::entity::allocator::Location
    pub(crate) fn new(indices: Range<usize>, identifier: archetype::IdentifierRef<R>) -> Self {
        Self {
            indices,
            identifier,
        }
    }

    /// The remaining number of [`Location`]s yet to be iterated.
    ///
    /// [`Location`]: crate::entity::allocator::Location
    fn len(&self) -> usize {
        debug_assert!(self.indices.end >= self.indices.start);
        self.indices.end - self.indices.start
    }

    /// Returns `true` if there are no more [`Location`]s to be iterated over.
    ///
    /// [`Location`]: crate::entity::allocator::Location
    fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

impl<R> Iterator for Locations<R>
where
    R: Registry,
{
    type Item = Location<R>;

    /// Returns the next [`Location`], if there are any left to be returned.
    ///
    /// Note that `Location`s are returned in sequential order, according to the `indices` provided
    /// on construction.
    ///
    /// [`Location`]: crate::entity::allocator::Location
    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|index| Location {
            identifier: self.identifier,
            index,
        })
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
