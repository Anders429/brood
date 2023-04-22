use crate::{
    component,
    entity,
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

pub trait Sealed<Entity, Indices>: Canonical<Self::Canonical, Self::CanonicalContainments> {
    /// The canonical form of the entity `E`.
    ///
    /// This type is guaranteed to be canonical with respect to the current registry, and that
    /// relationship is embodied by an impl of the `Canonical` trait on the current registry.
    type Canonical: entity::Entity;
    type CanonicalContainments;

    /// Returns the canonical form of the entity, consuming the original entity.
    fn canonical(entity: Entity) -> Self::Canonical;
}

impl<Registry, Entity, Containments, CanonicalContainments, Indices>
    Sealed<Entity, (Containments, CanonicalContainments, Indices)> for Registry
where
    Registry: Expanded<Entity, Containments, CanonicalContainments, Indices>,
{
    type Canonical = Registry::Canonical;
    type CanonicalContainments = CanonicalContainments;

    fn canonical(entity: Entity) -> Self::Canonical {
        Registry::canonical(entity)
    }
}

pub trait Expanded<Entity, Containments, CanonicalContainments, Indices>:
    Canonical<Self::Canonical, CanonicalContainments>
{
    /// The canonical form of the entity `E`.
    ///
    /// This type is guaranteed to be canonical with respect to the current registry, and that
    /// relationship is embodied by an impl of the `Canonical` trait on the current registry.
    type Canonical: entity::Entity;

    /// Returns the canonical form of the entity, consuming the original entity.
    fn canonical(entity: Entity) -> Self::Canonical;
}

impl<CanonicalContainments> Expanded<entity::Null, Null, CanonicalContainments, Null>
    for registry::Null
where
    Self: Canonical<entity::Null, CanonicalContainments>,
{
    type Canonical = entity::Null;

    fn canonical(_entity: entity::Null) -> Self::Canonical {
        entity::Null
    }
}

impl<
        Component,
        Registry,
        Entity,
        Containments,
        CanonicalContainment,
        CanonicalContainments,
        Index,
        Indices,
    >
    Expanded<
        Entity,
        (Contained, Containments),
        (CanonicalContainment, CanonicalContainments),
        (Index, Indices),
    > for (Component, Registry)
where
    Self: Canonical<
        (
            Component,
            <Registry as Expanded<
                <Entity as Get<Component, Index>>::Remainder,
                Containments,
                CanonicalContainments,
                Indices,
            >>::Canonical,
        ),
        (CanonicalContainment, CanonicalContainments),
    >,
    Registry: Expanded<
        <Entity as Get<Component, Index>>::Remainder,
        Containments,
        CanonicalContainments,
        Indices,
    >,
    Entity: Get<Component, Index>,
    Component: component::Component,
{
    type Canonical = (
        Component,
        <Registry as Expanded<
            <Entity as Get<Component, Index>>::Remainder,
            Containments,
            CanonicalContainments,
            Indices,
        >>::Canonical,
    );

    fn canonical(entity: Entity) -> Self::Canonical {
        let (component, remainder) = entity.get();
        (component, Registry::canonical(remainder))
    }
}

impl<C, E, I, P, CanonicalContainment, CanonicalContainments, R>
    Expanded<E, (NotContained, P), (CanonicalContainment, CanonicalContainments), I> for (C, R)
where
    Self: Canonical<
        <R as Expanded<E, P, CanonicalContainments, I>>::Canonical,
        (CanonicalContainment, CanonicalContainments),
    >,
    R: Expanded<E, P, CanonicalContainments, I>,
{
    type Canonical = <R as Expanded<E, P, CanonicalContainments, I>>::Canonical;

    fn canonical(entity: E) -> Self::Canonical {
        R::canonical(entity)
    }
}

#[cfg(test)]
mod entity_tests {
    use super::Sealed;
    use crate::{
        entity,
        Registry,
    };

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

    type Registry = Registry!(A, B, C, D, E);

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
