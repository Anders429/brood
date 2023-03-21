use crate::{
    registry,
    world::World,
};

impl<R, Resources> Clone for World<R, Resources>
where
    R: registry::Clone,
    Resources: Clone,
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

            resources: self.resources.clone(),
        }
    }

    /// Performs a full clone of `source` into `self`, cloning all of its components.
    ///
    /// Any `entity::Identifier`s that were valid for the `source` `World` will be valid for `self`
    /// after this. Old `entity::Identifier`s that were valid for `self` before this clone will no
    /// longer be valid.
    ///
    /// This method reuses the existing allocations for the clone. In some cases, this can be more
    /// efficient than calling `clone()` directly.
    fn clone_from(&mut self, source: &Self) {
        // SAFETY: `identifier_map` will be outlived by both the current and the source `World`,
        // and therefore will be outlived by the archetypes it references as well.
        let identifier_map = unsafe { self.archetypes.clone_from(&source.archetypes) };
        // SAFETY: `identifier_map` is guaranteed to contain an entry for every archetype in the
        // world, meaning there will be an entry for every archetype identifier referenced in
        // `self.entity_allocator`.
        unsafe {
            self.entity_allocator
                .clone_from(&source.entity_allocator, &identifier_map);
        }
        self.len = source.len;

        self.resources.clone_from(&source.resources);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        entities,
        entity,
        resources,
        Registry,
        Resources,
        World,
    };

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct A(u32);

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct B(char);

    type Registry = Registry!(A, B);
    type Resources = Resources!(A, B);

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

    #[test]
    fn clone_resources() {
        let world = World::<Registry, Resources>::with_resources(resources!(A(42), B('a')));

        assert_eq!(world, world.clone());
    }

    #[test]
    fn clone_components_and_resources() {
        let mut world = World::<Registry, Resources>::with_resources(resources!(A(42), B('a')));

        world.insert(entity!(A(42)));
        world.extend(entities!((B('a')); 5));
        world.extend(entities!((A(100), B('b')); 10));

        assert_eq!(world, world.clone());
    }

    #[test]
    fn clone_from_empty_into_empty() {
        let mut world = World::<Registry>::new();
        let source_world = World::<Registry>::new();

        world.clone_from(&source_world);

        assert_eq!(world, source_world);
    }

    #[test]
    fn clone_from_empty_into_nonempty() {
        let mut world = World::<Registry>::new();
        world.insert(entity!(A(42)));
        world.extend(entities!((B('a')); 5));
        world.extend(entities!((A(100), B('b')); 10));
        let source_world = World::<Registry>::new();

        world.clone_from(&source_world);

        // The two worlds are not considered equal, because `world` retains its archetypes. But the
        // two both have no components, because the data from `source_world` was cloned into
        // `world`.
        assert_ne!(world, source_world);
        assert_eq!(world.len(), source_world.len());
    }

    #[test]
    fn clone_from_nonempty_into_empty() {
        let mut world = World::<Registry>::new();
        let mut source_world = World::<Registry>::new();
        source_world.insert(entity!(A(42)));
        source_world.extend(entities!((B('a')); 5));
        source_world.extend(entities!((A(100), B('b')); 10));

        world.clone_from(&source_world);

        assert_eq!(world, source_world);
    }

    #[test]
    fn clone_from_nonempty_into_nonempty() {
        let mut world = World::<Registry>::new();
        world.insert(entity!(A(42)));
        world.extend(entities!((B('a')); 5));
        world.extend(entities!((A(100), B('b')); 10));
        let mut source_world = World::<Registry>::new();
        source_world.extend(entities!((A(100)); 10));
        source_world.insert(entity!(B('a')));
        source_world.insert(entity!(A(42), B('b')));

        world.clone_from(&source_world);

        assert_eq!(world, source_world);
    }
}
