use crate::{
    archetype,
    registry::Registry,
};
use by_address::ByThinAddress;
use core::{
    fmt,
    fmt::Debug,
};
use fnv::FnvBuildHasher;
use hashbrown::HashMap;

/// Defines an entity's location.
///
/// This is used by the entity allocator to map from an entity identifier to the actual entity.
pub(crate) struct Location<R>
where
    R: Registry,
{
    /// The identifier of the archetype currently storing this entity.
    pub(crate) identifier: archetype::IdentifierRef<R>,
    /// The index of the entity within its archetype.
    pub(crate) index: usize,
}

impl<R> Location<R>
where
    R: Registry,
{
    /// Creates a new location from an archetype identifier and an index within that archetype.
    pub(crate) fn new(identifier: archetype::IdentifierRef<R>, index: usize) -> Self {
        Self { identifier, index }
    }

    /// Clone using a new set of archetype identifiers.
    ///
    /// This is for cloning of an entire world, in which case the archetype identifiers will
    /// change.
    ///
    /// # Safety
    /// `identifier_map` must contain an entry for `self.identifier`.
    pub(super) unsafe fn clone_with_new_identifier(
        &self,
        identifier_map: &HashMap<ByThinAddress<&[u8]>, archetype::IdentifierRef<R>, FnvBuildHasher>,
    ) -> Self {
        Self {
            // SAFETY: `identifier_map` is guaranteed to contain an entry for `self.identifier`.
            identifier: *unsafe {
                identifier_map
                    .get(&ByThinAddress(self.identifier.as_slice()))
                    .unwrap_unchecked()
            },
            index: self.index,
        }
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

#[cfg(test)]
mod tests {
    use super::Location;
    use crate::{
        archetype::Identifier,
        Registry,
    };
    use alloc::vec;

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
        Registry!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

    #[test]
    fn new() {
        let identifier = unsafe { Identifier::<Registry>::new(vec![1, 2, 3, 0]) };
        let location = Location::new(unsafe { identifier.as_ref() }, 42);

        assert_eq!(location.identifier, unsafe { identifier.as_ref() });
        assert_eq!(location.index, 42);
    }
}
