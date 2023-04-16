use crate::{
    component,
    entities,
    hlist::Get,
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
pub trait Sealed<Entities, Indices>:
    Canonical<<Self::Canonical as entities::Contains>::Entity, Self::CanonicalContainments>
{
    /// The canonical form of the entities.
    ///
    /// This type is guaranteed to be canonical with respect to the current registry, and that
    /// relationship is embodied by an impl of the `Canonical` trait on the current registry.
    type Canonical: entities::Entities;
    type CanonicalContainments;

    /// Returns the canonical form of the entities, consuming the original entities.
    fn canonical(entities: Entities) -> Self::Canonical;
}

impl<Registry, Entities, Containments, CanonicalContainments, Indices>
    Sealed<Entities, (Containments, CanonicalContainments, Indices)> for Registry
where
    Registry: Expanded<Entities, Containments, CanonicalContainments, Indices>,
{
    type Canonical = Registry::Canonical;
    type CanonicalContainments = CanonicalContainments;

    fn canonical(entities: Entities) -> Self::Canonical {
        Registry::canonical(entities)
    }
}

pub trait Expanded<Entities, Containments, CanonicalContainments, Indices>:
    Canonical<<Self::Canonical as entities::Contains>::Entity, CanonicalContainments>
{
    /// The canonical form of the entities.
    ///
    /// This type is guaranteed to be canonical with respect to the current registry, and that
    /// relationship is embodied by an impl of the `Canonical` trait on the current registry.
    type Canonical: entities::Entities;

    /// Returns the canonical form of the entities, consuming the original entities.
    fn canonical(entities: Entities) -> Self::Canonical;
}

impl<CanonicalContainments> Expanded<entities::Null, Null, CanonicalContainments, Null>
    for registry::Null
where
    Self: Canonical<<entities::Null as entities::Contains>::Entity, CanonicalContainments>,
{
    type Canonical = entities::Null;

    fn canonical(_entities: entities::Null) -> Self::Canonical {
        entities::Null
    }
}

impl<
        Component,
        Registry,
        Entities,
        Containments,
        CanonicalContainment,
        CanonicalContainments,
        Index,
        Indices,
    >
    Expanded<
        Entities,
        (Contained, Containments),
        (CanonicalContainment, CanonicalContainments),
        (Index, Indices),
    > for (Component, Registry)
where
    Self: Canonical<
        <(
            Vec<Component>,
            <Registry as Expanded<
                <Entities as Get<Vec<Component>, Index>>::Remainder,
                Containments,
                CanonicalContainments,
                Indices,
            >>::Canonical,
        ) as entities::Contains>::Entity,
        (CanonicalContainment, CanonicalContainments),
    >,
    Registry: Expanded<
        <Entities as Get<Vec<Component>, Index>>::Remainder,
        Containments,
        CanonicalContainments,
        Indices,
    >,
    Entities: Get<Vec<Component>, Index>,
    Component: component::Component,
{
    type Canonical = (
        Vec<Component>,
        <Registry as Expanded<
            <Entities as Get<Vec<Component>, Index>>::Remainder,
            Containments,
            CanonicalContainments,
            Indices,
        >>::Canonical,
    );

    fn canonical(entities: Entities) -> Self::Canonical {
        let (component, remainder) = entities.get();
        (component, Registry::canonical(remainder))
    }
}

impl<Component, Registry, Entities, Containments, CanonicalContainment, CanonicalContainments, Indices> Expanded<Entities, (NotContained, Containments), (CanonicalContainment, CanonicalContainments), Indices> for (Component, Registry)
where
    Self: Canonical<<<Registry as Expanded<Entities, Containments, CanonicalContainments, Indices>>::Canonical as entities::Contains>::Entity, (CanonicalContainment, CanonicalContainments)>,
    Registry: Expanded<Entities, Containments, CanonicalContainments, Indices>,
{
    type Canonical = <Registry as Expanded<Entities, Containments, CanonicalContainments, Indices>>::Canonical;

    fn canonical(entities: Entities) -> Self::Canonical {
        Registry::canonical(entities)
    }
}

#[cfg(test)]
mod tests {
    use super::Sealed;
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
