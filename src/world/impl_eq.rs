use super::World;
use crate::registry::{RegistryEq, RegistryPartialEq};

impl<R> PartialEq for World<R>
where
    R: RegistryPartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.archetypes == other.archetypes && self.entity_allocator == other.entity_allocator
    }
}

impl<R> Eq for World<R> where R: RegistryEq {}

#[cfg(test)]
mod tests {
    // TODO
}
