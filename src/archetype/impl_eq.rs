use crate::{
    archetype::Archetype,
    registry::{
        RegistryEq,
        RegistryPartialEq,
    },
};
use alloc::vec::Vec;
use core::mem::ManuallyDrop;

impl<R> PartialEq for Archetype<R>
where
    R: RegistryPartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length
            && self.identifier == other.identifier
            && ManuallyDrop::new(
                // SAFETY: `self.entity_identifiers` is guaranteed to contain the raw parts for a
                // valid `Vec` of size `self.length`.
                unsafe {
                Vec::from_raw_parts(
                    self.entity_identifiers.0,
                    self.length,
                    self.entity_identifiers.1,
                )
            }) == ManuallyDrop::new(
                // SAFETY: `other.entity_identifiers` is guaranteed to contain the raw parts for a
                // valid `Vec` of size `other.length`.
                unsafe {
                Vec::from_raw_parts(
                    other.entity_identifiers.0,
                    other.length,
                    other.entity_identifiers.1,
                )
            })
            &&
            // SAFETY: Since `self.identifier` is equal to `other.identifier`, the components Vecs
            // will contain the same number of values as there are bits in `self.identifier`.
            //
            // `self.components` and `other.components` both contain raw parts for valid `Vec<C>`s
            // for each identified component `C` of size `self.length` (since `self.length` and
            // `other.length` are equal).
            //
            // `self.identifier` is generic over the same `R` upon which this function is being
            // called.
            unsafe {
                R::component_eq(
                    &self.components,
                    &other.components,
                    self.length,
                    self.identifier.iter(),
                )
            }
    }
}

impl<R> Eq for Archetype<R> where R: RegistryEq {}
