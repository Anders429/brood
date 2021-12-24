use crate::internal::{
    archetype::Archetype,
    registry::{RegistryPartialEq, RegistryEq},
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
            && ManuallyDrop::new(unsafe {Vec::from_raw_parts(self.entity_identifiers.0, self.length, self.entity_identifiers.1)}) == ManuallyDrop::new(unsafe {Vec::from_raw_parts(other.entity_identifiers.0, other.length, other.entity_identifiers.1)})
            && unsafe { R::component_eq(&self.components, &other.components, self.length, self.identifier.as_identifier().as_slice(), 0, 0) }
    }
}

impl<R> Eq for Archetype<R> where R: RegistryEq {}
