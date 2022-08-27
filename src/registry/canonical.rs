//! Defines a canonical ordering for a registry.
//! 
//! This allows entities made of components within a registry to be reordered in the same order as
//! the components in the registry (i.e. the "canonical order" of components). Reordering
//! components in the canonical ordering of a registry means that the type signature for entities
//! will be the same for any entity made of the same components. This allows entities to be defined
//! in any order and to easily be reordered internally to a canonical order.
//! 
//! Attempting to reorder an entity containing components not in the registry will fail to compile.

use crate::{entity, entity::{Entity, Get}, registry, component::Component};

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
pub trait Canonical<E, P, I> {
    /// The canonical form of the entity `E`.
    type Canonical: Entity;

    /// Returns the canonical form of the entity, consuming the original entity.
    fn canonical(entity: E) -> Self::Canonical;
}

impl Canonical<entity::Null, Null, Null> for registry::Null {
    type Canonical = entity::Null;

    fn canonical(_entity: entity::Null) -> Self::Canonical {
        entity::Null
    }
}

impl<C, E, I, P, R, IS> Canonical<E, (Contained, P), (I, IS)> for (C, R)
where
    R: Canonical<<E as Get<C, I>>::Remainder, P, IS>,
    E: Get<C, I>,
    C: Component,
{
    type Canonical = (C, <R as Canonical<<E as Get<C, I>>::Remainder, P, IS>>::Canonical);

    fn canonical(entity: E) -> Self::Canonical {
        let (component, remainder) = entity.get();
        (component, R::canonical(remainder))
    }
}

impl<C, E, I, P, R> Canonical<E, (NotContained, P), I> for (C, R)
where
    R: Canonical<E, P, I>,
{
    type Canonical = <R as Canonical<E, P, I>>::Canonical;
    
    fn canonical(entity: E) -> Self::Canonical {
        R::canonical(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(Registry::canonical(entity!(A, B, C, D, E)), entity!(A, B, C, D, E));
    }

    #[test]
    fn entity_all_components_reordered() {
        assert_eq!(Registry::canonical(entity!(D, B, A, E, C)), entity!(A, B, C, D, E));
    }
}
