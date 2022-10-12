use crate::{
    archetype,
    entity::allocator::Location,
    registry::Registry,
};
use core::ops::Range;

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
    pub(super) fn len(&self) -> usize {
        debug_assert!(self.indices.end >= self.indices.start);
        self.indices.end - self.indices.start
    }

    /// Returns `true` if there are no more [`Location`]s to be iterated over.
    ///
    /// [`Location`]: crate::entity::allocator::Location
    pub(super) fn is_empty(&self) -> bool {
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
