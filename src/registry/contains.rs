use crate::{
    component::Component,
    entities,
    entities::{Entities, Get},
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
    R: ContainsEntities<<E as Get<C, I>>::Remainder, P, IS>,
    E: Get<C, I>,
    C: Component,
{
    type Canonical = (
        Vec<C>,
        <R as ContainsEntities<<E as Get<C, I>>::Remainder, P, IS>>::Canonical,
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
mod tests {
    use super::*;
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
    fn entity_empty() {
        assert_eq!(Registry::canonical(entities!((); 100).entities), entities!((); 100).entities);
    }

    #[test]
    fn entity_subset() {
        assert_eq!(Registry::canonical(entities!((E, C, B); 100).entities), entities!((B, C, E); 100).entities);
    }

    #[test]
    fn entity_all_components_already_canonical_order() {
        assert_eq!(
            Registry::canonical(entities!((A, B, C, D, E); 100).entities),
            entities!((A, B, C, D, E); 100).entities
        );
    }

    #[test]
    fn entity_all_components_reordered() {
        assert_eq!(
            Registry::canonical(entities!((D, B, A, E, C); 100).entities),
            entities!((A, B, C, D, E); 100).entities
        );
    }
}
