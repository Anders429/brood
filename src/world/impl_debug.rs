use super::World;
use crate::registry;
use core::fmt;

impl<R, Resources> fmt::Debug for World<R, Resources>
where
    R: registry::Debug,
    Resources: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("archetypes", &self.archetypes)
            .field("entity_allocator", &self.entity_allocator)
            .field("len", &self.len)
            .field("resources", &self.resources)
            .finish()
    }
}
