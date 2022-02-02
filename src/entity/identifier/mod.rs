#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;

/// A unique identifier for an entity.
///
/// An `Identifier` can be used to reference an entity that is stored within a [`World`].
/// `Identifier`s are normally obtained when inserting an entity into a `World`, although they can
/// also be obtained through a [`query`] on a `World` by providing `Identifier` as a [`View`].
///
/// # Example
/// ``` rust
/// use brood::{entity, registry, World};
///
/// // Define components.
/// struct Foo(usize);
/// struct Bar(bool);
///
/// type Registry = registry!(Foo, Bar);
///
/// let mut world = World::<Registry>::new();
///
/// // An identifier is returned on insertion.
/// let entity_identifier = world.push(entity!(Foo(42), Bar(false)));
/// ```
///
/// [`query`]: crate::world::World::query()
/// [`View`]: crate::query::view::View
/// [`World`]: crate::world::World
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Identifier {
    pub(crate) index: usize,
    pub(crate) generation: u64,
}

impl Identifier {
    pub(crate) fn new(index: usize, generation: u64) -> Self {
        Self { index, generation }
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::Identifier;

    #[test]
    fn new_index() {
        let identifier = Identifier::new(1, 2);

        assert_eq!(identifier.index, 1);
    }

    #[test]
    fn new_generation() {
        let identifier = Identifier::new(1, 2);

        assert_eq!(identifier.generation, 2);
    }
}
