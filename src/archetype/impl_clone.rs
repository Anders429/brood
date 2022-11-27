use crate::{
    archetype::Archetype,
    registry,
};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

impl<R> Clone for Archetype<R>
where
    R: registry::Clone,
{
    fn clone(&self) -> Self {
        let identifier = self.identifier.clone();

        // SAFETY: `self.entity_identifiers` is guaranteed to contain the raw parts for a valid
        // `Vec` of size `self.length`.
        let entity_identifiers = ManuallyDrop::new(unsafe {
            Vec::from_raw_parts(
                self.entity_identifiers.0,
                self.length,
                self.entity_identifiers.1,
            )
        });
        let mut cloned_entity_identifiers = entity_identifiers.clone();

        Self {
            identifier,

            entity_identifiers: (
                cloned_entity_identifiers.as_mut_ptr(),
                cloned_entity_identifiers.capacity(),
            ),
            // SAFETY: `self.components` contains the valid raw parts for a `Vec<C>` for each `C`
            // identified by `self.identifier`, with length `self.length`. The `R` upon which this
            // function is called is the same `R` that `self.identifier` is generic over.
            components: unsafe {
                R::clone_components(
                    &self.components,
                    Vec::with_capacity(self.components.len()),
                    self.length,
                    self.identifier.iter(),
                )
            },
            length: self.length,
        }
    }
}
