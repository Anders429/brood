use crate::{
    registry,
    world::World,
};

impl<R> Clone for World<R>
where
    R: registry::Clone,
{
    /// Performs a full clone of the `World` and all of its components.
    ///
    /// Any `entity::Identifier`s that were valid for the old `World` will be valid for the
    /// newly-cloned `World`.
    fn clone(&self) -> Self {
        // SAFETY: `identifier_map` will be outlived by both the current and the cloned `World`,
        // and therefore will be outlived by the archetypes it references as well.
        let (archetypes, identifier_map) = unsafe { self.archetypes.clone() };

        Self {
            archetypes,
            // SAFETY: `identifier_map` is guaranteed to contain an entry for every archetype in
            // the world, meaning there will be an entry for every archetype identifier referenced
            // in `self.entity_allocator`.
            entity_allocator: unsafe { self.entity_allocator.clone(&identifier_map) },
            len: self.len,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        entities,
        entity,
        Registry,
        World,
    };

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct A(u32);

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct B(char);

    type Registry = Registry!(A, B);

    #[test]
    fn clone_empty() {
        let world = World::<Registry>::new();

        assert_eq!(world, world.clone());
    }

    #[test]
    fn clone_components() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(42)));
        world.extend(entities!((B('a')); 5));
        world.extend(entities!((A(100), B('b')); 10));

        assert_eq!(world, world.clone());
    }

    #[test]
    fn clone_components_after_remove() {
        let mut world = World::<Registry>::new();

        let entity_identifier = world.insert(entity!(A(42)));
        world.insert(entity!(B('a')));
        world.remove(entity_identifier);

        assert_eq!(world, world.clone());
    }

    #[test]
    fn clone_after_shrink() {
        let mut world = World::<Registry>::new();

        world.insert(entity!(A(42)));
        world.extend(entities!((B('a')); 5));
        world.extend(entities!((A(100), B('b')); 10));
        world.clear();
        world.shrink_to_fit();
        world.insert(entity!(B('a')));

        assert_eq!(world, world.clone());
    }
}
