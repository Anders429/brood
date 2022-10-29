use super::World;
use crate::registry::RegistryDebug;
use core::fmt::{
    self,
    Debug,
};

impl<R> Debug for World<R>
where
    R: RegistryDebug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("archetypes", &self.archetypes)
            .field("entity_allocator", &self.entity_allocator)
            .finish()
    }
}
