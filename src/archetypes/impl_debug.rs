use crate::{archetypes::Archetypes, registry::RegistryDebug};
use core::{fmt, fmt::Debug};

impl<R> Debug for Archetypes<R>
where
    R: RegistryDebug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(
                self.iter()
                    .map(|archetype| (unsafe { archetype.identifier() }, archetype)),
            )
            .finish()
    }
}
