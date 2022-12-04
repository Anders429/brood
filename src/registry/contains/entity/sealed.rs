use crate::{
    component::Component,
    entity,
    entity::Entity,
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

pub trait Sealed<E, P, Q, I>: Canonical<Self::Canonical, Q> {
    /// The canonical form of the entity `E`.
    ///
    /// This type is guaranteed to be canonical with respect to the current registry, and that
    /// relationship is embodied by an impl of the `Canonical` trait on the current registry.
    type Canonical: Entity;

    /// Returns the canonical form of the entity, consuming the original entity.
    fn canonical(entity: E) -> Self::Canonical;
}

impl<Q> Sealed<entity::Null, Null, Q, Null> for registry::Null
where
    Self: Canonical<entity::Null, Q>,
{
    type Canonical = entity::Null;

    fn canonical(_entity: entity::Null) -> Self::Canonical {
        entity::Null
    }
}

impl<C, E, I, P, Q, QS, R, IS> Sealed<E, (Contained, P), (Q, QS), (I, IS)> for (C, R)
where
    Self: Canonical<
        (
            C,
            <R as Sealed<<E as entity::Get<C, I>>::Remainder, P, QS, IS>>::Canonical,
        ),
        (Q, QS),
    >,
    R: Sealed<<E as entity::Get<C, I>>::Remainder, P, QS, IS>,
    E: entity::Get<C, I>,
    C: Component,
{
    type Canonical = (
        C,
        <R as Sealed<<E as entity::Get<C, I>>::Remainder, P, QS, IS>>::Canonical,
    );

    fn canonical(entity: E) -> Self::Canonical {
        let (component, remainder) = entity.get();
        (component, R::canonical(remainder))
    }
}

impl<C, E, I, P, Q, QS, R> Sealed<E, (NotContained, P), (Q, QS), I> for (C, R)
where
    Self: Canonical<<R as Sealed<E, P, QS, I>>::Canonical, (Q, QS)>,
    R: Sealed<E, P, QS, I>,
{
    type Canonical = <R as Sealed<E, P, QS, I>>::Canonical;

    fn canonical(entity: E) -> Self::Canonical {
        R::canonical(entity)
    }
}

#[cfg(test)]
mod entity_tests {
    use super::*;
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
