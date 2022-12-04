use crate::{
    archetypes::Archetypes,
    registry,
};
use core::{
    fmt,
    fmt::Debug,
};

impl<R> Debug for Archetypes<R>
where
    R: registry::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|archetype| {
                (
                    // SAFETY: The `IdentifierRef` obtained here does not live longer than the
                    // `archetype`.
                    unsafe { archetype.identifier() },
                    archetype,
                )
            }))
            .finish()
    }
}
