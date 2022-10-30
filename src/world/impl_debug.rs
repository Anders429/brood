use super::World;
use crate::registry;
use core::fmt;

impl<R> fmt::Debug for World<R>
where
    R: registry::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("archetypes", &self.archetypes)
            .field("entity_allocator", &self.entity_allocator)
            .finish()
    }
}
