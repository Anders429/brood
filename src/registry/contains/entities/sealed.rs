use crate::{
    component::Component,
    entities,
    entities::Entities,
    registry,
    registry::{
        contains::{
            Contained,
            NotContained,
            Null,
        },
        Canonical,
    },
};
use alloc::vec::Vec;

/// Indicates that all of an entities' components are contained in the registry.
///
/// This allows reordering the components of the entities into the canonical ordering defined by
/// the registry.
///
/// If the entities contain components not in this registry, attempting to use this trait will
/// result in a compiler error, since the trait won't be implemented for the combination of entity
/// and registry.
///
/// This is generic over entities `E`, containments `P` (indicating whether each component is
/// contained in the registry), and indices `I` (indicating the location of each component in the
/// entity `E`).
pub trait Sealed<E, P, Q, I>:
    Canonical<<Self::Canonical as entities::Contains>::Entity, Q>
{
    /// The canonical form of the entity `E`.
    ///
    /// This type is guaranteed to be canonical with respect to the current registry, and that
    /// relationship is embodied by an impl of the `Canonical` trait on the current registry.
    type Canonical: Entities;

    /// Returns the canonical form of the entities, consuming the original entities.
    fn canonical(entities: E) -> Self::Canonical;
}

impl<Q> Sealed<entities::Null, Null, Q, Null> for registry::Null
where
    Self: Canonical<<entities::Null as entities::Contains>::Entity, Q>,
{
    type Canonical = entities::Null;

    fn canonical(_entities: entities::Null) -> Self::Canonical {
        entities::Null
    }
}

impl<C, E, I, P, Q, QS, R, IS> Sealed<E, (Contained, P), (Q, QS), (I, IS)> for (C, R)
where
    Self: Canonical<
        <(
            Vec<C>,
            <R as Sealed<<E as entities::Get<C, I>>::Remainder, P, QS, IS>>::Canonical,
        ) as entities::Contains>::Entity,
        (Q, QS),
    >,
    R: Sealed<<E as entities::Get<C, I>>::Remainder, P, QS, IS>,
    E: entities::Get<C, I>,
    C: Component,
{
    type Canonical = (
        Vec<C>,
        <R as Sealed<<E as entities::Get<C, I>>::Remainder, P, QS, IS>>::Canonical,
    );

    fn canonical(entities: E) -> Self::Canonical {
        let (component, remainder) = entities.get();
        (component, R::canonical(remainder))
    }
}

impl<C, E, I, P, Q, QS, R> Sealed<E, (NotContained, P), (Q, QS), I> for (C, R)
where
    Self: Canonical<<<R as Sealed<E, P, QS, I>>::Canonical as entities::Contains>::Entity, (Q, QS)>,
    R: Sealed<E, P, QS, I>,
{
    type Canonical = <R as Sealed<E, P, QS, I>>::Canonical;

    fn canonical(entities: E) -> Self::Canonical {
        R::canonical(entities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        entities,
        Registry,
    };

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

    type Registry = Registry!(A, B, C, D, E);

    #[test]
    fn entities_empty() {
        assert_eq!(
            Registry::canonical(entities!((); 100).entities),
            entities!((); 100).entities
        );
    }

    #[test]
    fn entities_subset() {
        assert_eq!(
            Registry::canonical(entities!((E, C, B); 100).entities),
            entities!((B, C, E); 100).entities
        );
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
