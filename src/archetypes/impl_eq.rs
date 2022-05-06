use crate::{
    archetypes::Archetypes,
    registry::{RegistryEq, RegistryPartialEq},
};

impl<R> PartialEq for Archetypes<R>
where
    R: RegistryPartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.raw_archetypes.len() != other.raw_archetypes.len() {
            return false;
        }

        self.iter().all(|archetype| {
            other
                .get(
                    // SAFETY: The `IdentifierRef` obtained here does not live longer than the
                    // `archetype`.
                    unsafe { archetype.identifier() },
                )
                .map_or(false, |other_archetype| archetype == other_archetype)
        })
    }
}

impl<R> Eq for Archetypes<R> where R: RegistryEq {}
