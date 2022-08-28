use crate::{
    component::Component,
    entities,
    entities::Entities,
    entity,
    entity::Entity,
    registry,
};
use alloc::vec::Vec;

/// Type marker for a component contained in an entity.
///
/// This does not have to be specified when calling `canonical()`. The compiler can infer its
/// value.
pub enum Contained {}

/// Type marker for a component not contained in an entity.
///
/// This does not have to be specified when calling `canonical()`. The compiler can infer its
/// value.
pub enum NotContained {}

/// Defines either a null position or a null index.
///
/// This does not have to be specified when calling `canonical()`. The compiler can infer its
/// value.
pub enum Null {}

/// Converts an entity to the canonical form defined by this registry.
///
/// If the entity contains components not in this registry, attempting to use this trait will
/// result in a compiler error, since the trait won't be implemented for the combination of entity
/// and registry.
///
/// This is generic over an entity `E`, containments `P` (indicating whether each component is
/// contained in the registry), and indices `I` (indicating the location of each component in the
/// entity `E`).
pub trait ContainsEntity<E, P, I> {
    /// The canonical form of the entity `E`.
    type Canonical: Entity;

    /// Returns the canonical form of the entity, consuming the original entity.
    fn canonical(entity: E) -> Self::Canonical;
}

impl ContainsEntity<entity::Null, Null, Null> for registry::Null {
    type Canonical = entity::Null;

    fn canonical(_entity: entity::Null) -> Self::Canonical {
        entity::Null
    }
}

impl<C, E, I, P, R, IS> ContainsEntity<E, (Contained, P), (I, IS)> for (C, R)
where
    R: ContainsEntity<<E as entity::Get<C, I>>::Remainder, P, IS>,
    E: entity::Get<C, I>,
    C: Component,
{
    type Canonical = (
        C,
        <R as ContainsEntity<<E as entity::Get<C, I>>::Remainder, P, IS>>::Canonical,
    );

    fn canonical(entity: E) -> Self::Canonical {
        let (component, remainder) = entity.get();
        (component, R::canonical(remainder))
    }
}

impl<C, E, I, P, R> ContainsEntity<E, (NotContained, P), I> for (C, R)
where
    R: ContainsEntity<E, P, I>,
{
    type Canonical = <R as ContainsEntity<E, P, I>>::Canonical;

    fn canonical(entity: E) -> Self::Canonical {
        R::canonical(entity)
    }
}

/// Converts an entity to the canonical form defined by this registry.
///
/// If the entity contains components not in this registry, attempting to use this trait will
/// result in a compiler error, since the trait won't be implemented for the combination of entity
/// and registry.
///
/// This is generic over an entity `E`, containments `P` (indicating whether each component is
/// contained in the registry), and indices `I` (indicating the location of each component in the
/// entity `E`).
pub trait ContainsEntities<E, P, I> {
    /// The canonical form of the entity `E`.
    type Canonical: Entities;

    /// Returns the canonical form of the entity, consuming the original entity.
    fn canonical(entities: E) -> Self::Canonical;
}

impl ContainsEntities<entities::Null, Null, Null> for registry::Null {
    type Canonical = entities::Null;

    fn canonical(_entities: entities::Null) -> Self::Canonical {
        entities::Null
    }
}

impl<C, E, I, P, R, IS> ContainsEntities<E, (Contained, P), (I, IS)> for (C, R)
where
    R: ContainsEntities<<E as entities::Get<C, I>>::Remainder, P, IS>,
    E: entities::Get<C, I>,
    C: Component,
{
    type Canonical = (
        Vec<C>,
        <R as ContainsEntities<<E as entities::Get<C, I>>::Remainder, P, IS>>::Canonical,
    );

    fn canonical(entities: E) -> Self::Canonical {
        let (component, remainder) = entities.get();
        (component, R::canonical(remainder))
    }
}

impl<C, E, I, P, R> ContainsEntities<E, (NotContained, P), I> for (C, R)
where
    R: ContainsEntities<E, P, I>,
{
    type Canonical = <R as ContainsEntities<E, P, I>>::Canonical;

    fn canonical(entities: E) -> Self::Canonical {
        R::canonical(entities)
    }
}

#[cfg(test)]
mod entity_tests {
    use super::ContainsEntity;
    use crate::{entity, registry};

    #[derive(Debug, Eq, PartialEq)]
    struct A;
    #[derive(Debug, Eq, PartialEq)]
    struct B;
    #[derive(Debug, Eq, PartialEq)]
    struct C;
    #[derive(Debug, Eq, PartialEq)]
    struct D;
    #[derive(Debug, Eq, PartialEq)]
    struct E;

    type Registry = registry!(A, B, C, D, E);

    #[test]
    fn entity_empty() {
        assert_eq!(Registry::canonical(entity!()), entity!());
    }

    #[test]
    fn entity_subset() {
        assert_eq!(Registry::canonical(entity!(E, C, B)), entity!(B, C, E));
    }

    #[test]
    fn entity_all_components_already_canonical_order() {
        assert_eq!(
            Registry::canonical(entity!(A, B, C, D, E)),
            entity!(A, B, C, D, E)
        );
    }

    #[test]
    fn entity_all_components_reordered() {
        assert_eq!(
            Registry::canonical(entity!(D, B, A, E, C)),
            entity!(A, B, C, D, E)
        );
    }
}


#[cfg(test)]
mod entities_tests {
    use super::ContainsEntities;
    use crate::{entities, registry};

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct A;
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct B;
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct C;
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct D;
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct E;

    type Registry = registry!(A, B, C, D, E);

    #[test]
    fn entities_empty() {
        assert_eq!(Registry::canonical(entities!((); 100).entities), entities!((); 100).entities);
    }

    #[test]
    fn entities_subset() {
        assert_eq!(Registry::canonical(entities!((E, C, B); 100).entities), entities!((B, C, E); 100).entities);
    }

    #[test]
    fn entities_all_components_already_canonical_order() {
        assert_eq!(
            Registry::canonical(entities!((A, B, C, D, E); 100).entities),
            entities!((A, B, C, D, E); 100).entities
        );
    }

    #[test]
    fn entities_all_components_reordered() {
        assert_eq!(
            Registry::canonical(entities!((D, B, A, E, C); 100).entities),
            entities!((A, B, C, D, E); 100).entities
        );
    }
}
