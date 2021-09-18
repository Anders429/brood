use crate::internal::{archetype::Archetype, entity::EntityDebug};
use core::fmt::{self, Debug};

impl<E> Debug for Archetype<E>
where
    E: EntityDebug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { E::debug(&self.components, self.length, &mut f.debug_map()) }.finish()
    }
}
