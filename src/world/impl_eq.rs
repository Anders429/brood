use super::World;
use crate::registry;
use core::cmp;

impl<R, Resources> cmp::PartialEq for World<R, Resources>
where
    R: registry::PartialEq,
    Resources: cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len
            && self.archetypes == other.archetypes
            && self.entity_allocator == other.entity_allocator
            && self.resources == other.resources
    }
}

impl<R, Resources> cmp::Eq for World<R, Resources>
where
    R: registry::Eq,
    Resources: cmp::Eq,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        entity,
        resources,
        Registry,
        Resources,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct A(u32);

    #[derive(Debug, Eq, PartialEq)]
    struct B(char);

    type Registry = Registry!(A, B);
    type Resources = Resources!(A, B);

    #[test]
    fn empty_eq() {
        assert_eq!(World::<Registry!()>::new(), World::<Registry!()>::new());
    }

    #[test]
    fn with_entities_eq() {
        let mut world_a = World::<Registry>::new();
        let mut world_b = World::<Registry>::new();

        world_a.insert(entity!(A(1), B('a')));
        world_a.insert(entity!(A(2), B('b')));
        world_a.insert(entity!(A(3), B('c')));
        world_a.insert(entity!(A(4)));
        world_a.insert(entity!(A(5)));
        world_a.insert(entity!());

        world_b.insert(entity!(A(1), B('a')));
        world_b.insert(entity!(A(2), B('b')));
        world_b.insert(entity!(A(3), B('c')));
        world_b.insert(entity!(A(4)));
        world_b.insert(entity!(A(5)));
        world_b.insert(entity!());

        assert_eq!(world_a, world_b);
    }

    #[test]
    fn with_resources_eq() {
        let world_a = World::<Registry, Resources>::with_resources(resources!(A(42), B('a')));
        let world_b = World::<Registry, Resources>::with_resources(resources!(A(42), B('a')));

        assert_eq!(world_a, world_b);
    }

    #[test]
    fn with_entities_and_resources_eq() {
        let mut world_a = World::<Registry, Resources>::with_resources(resources!(A(42), B('a')));
        let mut world_b = World::<Registry, Resources>::with_resources(resources!(A(42), B('a')));

        world_a.insert(entity!(A(1), B('a')));
        world_a.insert(entity!(A(2), B('b')));
        world_a.insert(entity!(A(3), B('c')));
        world_a.insert(entity!(A(4)));
        world_a.insert(entity!(A(5)));
        world_a.insert(entity!());

        world_b.insert(entity!(A(1), B('a')));
        world_b.insert(entity!(A(2), B('b')));
        world_b.insert(entity!(A(3), B('c')));
        world_b.insert(entity!(A(4)));
        world_b.insert(entity!(A(5)));
        world_b.insert(entity!());

        assert_eq!(world_a, world_b);
    }

    #[test]
    fn archetypes_not_equal() {
        let mut world_a = World::<Registry>::new();
        let mut world_b = World::<Registry>::new();

        world_a.insert(entity!(A(1), B('a')));
        world_a.insert(entity!(A(2), B('b')));
        world_a.insert(entity!(A(3), B('c')));

        world_b.insert(entity!(A(1)));
        world_b.insert(entity!(A(2)));
        world_b.insert(entity!(A(3)));

        assert_ne!(world_a, world_b);
    }

    #[test]
    fn allocators_not_equal() {
        let mut world_a = World::<Registry>::new();
        let mut world_b = World::<Registry>::new();

        world_a.insert(entity!(A(1), B('a')));

        let entity_identifier = world_b.insert(entity!(A(1), B('a')));
        world_b.remove(entity_identifier);
        world_b.insert(entity!(A(1), B('a')));

        // The generational index of the entities will be different.
        assert_ne!(world_a, world_b);
    }

    #[test]
    fn resources_not_equal() {
        let world_a = World::<Registry, Resources>::with_resources(resources!(A(42), B('a')));
        let world_b = World::<Registry, Resources>::with_resources(resources!(A(42), B('b')));

        assert_ne!(world_a, world_b);
    }
}
