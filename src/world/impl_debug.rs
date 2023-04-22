use super::World;
use crate::{
    registry,
    resource,
};
use core::fmt;

impl<Registry, Resources> fmt::Debug for World<Registry, Resources>
where
    Registry: registry::Debug,
    Resources: resource::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field("archetypes", &self.archetypes)
            .field("entity_allocator", &self.entity_allocator)
            .field("len", &self.len)
            .field("resources", &resource::Debugger(&self.resources))
            .finish()
    }
}
