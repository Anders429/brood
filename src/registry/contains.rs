//! Traits indicating that the components in a heterogeneous list are contained within a registry.
//!
//! These traits are implemented for different kinds of heterogeneous lists (entity, entities,
//! etc.). They allow for reordering the components within that heterogeneous list in the same
//! order as the components in the registry, also known as the "canonical order".

#[cfg(feature = "rayon")]
use crate::query::view::ParViews;
use crate::{
    archetype,
    component::Component,
    entities,
    entities::Entities,
    entity,
    entity::Entity,
    query::{
        result::Reshape,
        view::{self, Views, ViewsSeal},
    },
    registry,
    registry::{Canonical, CanonicalViews, Length, Registry},
};
use alloc::vec::Vec;
use core::slice;

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

/// Defines either a null containment or a null index.
///
/// This does not have to be specified when calling `canonical()`. The compiler can infer its
/// value.
pub enum Null {}

/// Indicates that the component `C` is contained in the registry.
pub trait ContainsComponent<C, I> {
    /// Defines the index of the heterogeneous list where the component is located.
    ///
    /// Note that this is likely the opposite of what you want, since the last component has the
    /// index 0. To get the reverse of this, use `R::LEN - R::INDEX - 1`.
    const INDEX: usize;
}

impl<C, R> ContainsComponent<C, Contained> for (C, R)
where
    R: Length,
{
    const INDEX: usize = R::LEN;
}

impl<C, C_, I, R> ContainsComponent<C_, (I,)> for (C, R)
where
    R: ContainsComponent<C_, I>,
{
    const INDEX: usize = R::INDEX;
}

/// Indicates that all of an entity's components are contained in the registry.
///
/// This allows reordering the components of the entity into the canonical ordering defined by the
/// registry.
///
/// If the entity contains components not in this registry, attempting to use this trait will
/// result in a compiler error, since the trait won't be implemented for the combination of entity
/// and registry.
///
/// This is generic over an entity `E`, containments `P` (indicating whether each component is
/// contained in the registry), and indices `I` (indicating the location of each component in the
/// entity `E`).
pub trait ContainsEntity<E, P, Q, I>: Canonical<Self::Canonical, Q> {
    /// The canonical form of the entity `E`.
    ///
    /// This type is guaranteed to be canonical with respect to the current registry, and that
    /// relationship is embodied by an impl of the `Canonical` trait on the current registry.
    type Canonical: Entity;

    /// Returns the canonical form of the entity, consuming the original entity.
    fn canonical(entity: E) -> Self::Canonical;
}

impl<Q> ContainsEntity<entity::Null, Null, Q, Null> for registry::Null
where
    Self: Canonical<entity::Null, Q>,
{
    type Canonical = entity::Null;

    fn canonical(_entity: entity::Null) -> Self::Canonical {
        entity::Null
    }
}

impl<C, E, I, P, Q, QS, R, IS> ContainsEntity<E, (Contained, P), (Q, QS), (I, IS)> for (C, R)
where
    Self: Canonical<
        (
            C,
            <R as ContainsEntity<<E as entity::Get<C, I>>::Remainder, P, QS, IS>>::Canonical,
        ),
        (Q, QS),
    >,
    R: ContainsEntity<<E as entity::Get<C, I>>::Remainder, P, QS, IS>,
    E: entity::Get<C, I>,
    C: Component,
{
    type Canonical = (
        C,
        <R as ContainsEntity<<E as entity::Get<C, I>>::Remainder, P, QS, IS>>::Canonical,
    );

    fn canonical(entity: E) -> Self::Canonical {
        let (component, remainder) = entity.get();
        (component, R::canonical(remainder))
    }
}

impl<C, E, I, P, Q, QS, R> ContainsEntity<E, (NotContained, P), (Q, QS), I> for (C, R)
where
    Self: Canonical<<R as ContainsEntity<E, P, QS, I>>::Canonical, (Q, QS)>,
    R: ContainsEntity<E, P, QS, I>,
{
    type Canonical = <R as ContainsEntity<E, P, QS, I>>::Canonical;

    fn canonical(entity: E) -> Self::Canonical {
        R::canonical(entity)
    }
}

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
pub trait ContainsEntities<E, P, Q, I>:
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

impl<Q> ContainsEntities<entities::Null, Null, Q, Null> for registry::Null
where
    Self: Canonical<<entities::Null as entities::Contains>::Entity, Q>,
{
    type Canonical = entities::Null;

    fn canonical(_entities: entities::Null) -> Self::Canonical {
        entities::Null
    }
}

impl<C, E, I, P, Q, QS, R, IS> ContainsEntities<E, (Contained, P), (Q, QS), (I, IS)> for (C, R)
where
    Self: Canonical<
        <(
            Vec<C>,
            <R as ContainsEntities<<E as entities::Get<C, I>>::Remainder, P, QS, IS>>::Canonical,
        ) as entities::Contains>::Entity,
        (Q, QS),
    >,
    R: ContainsEntities<<E as entities::Get<C, I>>::Remainder, P, QS, IS>,
    E: entities::Get<C, I>,
    C: Component,
{
    type Canonical = (
        Vec<C>,
        <R as ContainsEntities<<E as entities::Get<C, I>>::Remainder, P, QS, IS>>::Canonical,
    );

    fn canonical(entities: E) -> Self::Canonical {
        let (component, remainder) = entities.get();
        (component, R::canonical(remainder))
    }
}

impl<C, E, I, P, Q, QS, R> ContainsEntities<E, (NotContained, P), (Q, QS), I> for (C, R)
where
    Self: Canonical<
        <<R as ContainsEntities<E, P, QS, I>>::Canonical as entities::Contains>::Entity,
        (Q, QS),
    >,
    R: ContainsEntities<E, P, QS, I>,
{
    type Canonical = <R as ContainsEntities<E, P, QS, I>>::Canonical;

    fn canonical(entities: E) -> Self::Canonical {
        R::canonical(entities)
    }
}

pub enum EntityIdentifierMarker {}

pub trait ContainsViews<'a, V, P, I, Q>
where
    V: Views<'a>,
{
    type Canonical: Views<'a> + ViewsSeal<'a, Results = Self::CanonicalResults>;
    type CanonicalResults: Reshape<V::Results, Q>;

    /// # Safety
    /// 
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`, ordered for the archetype identified by `archetype_identifier`.
    /// 
    /// Additionally, `entity_identifiers` must contain the raw parts for a valid
    /// `Vec<entity::Identifier>` of length `length`.
    unsafe fn view<R>(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R>,
    ) -> Self::CanonicalResults
    where
        R: Registry;
}

impl<'a, I, IS, P, V, R, Q> ContainsViews<'a, V, (Contained, P), (I, IS), Q>
    for (EntityIdentifierMarker, R)
where
    R: CanonicalViews<
        'a,
        <R as ContainsViewsInner<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
        P,
    > + ContainsViewsInner<'a, <V as view::Get<'a, entity::Identifier, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, entity::Identifier, I>,
    <(
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    ) as ViewsSeal<'a>>::Results: Reshape<<V as ViewsSeal<'a>>::Results, Q>,
{
    type Canonical = (
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
    type CanonicalResults = <Self::Canonical as ViewsSeal<'a>>::Results;

    unsafe fn view<R_>(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> Self::CanonicalResults
    where
        R_: Registry,
    {
        (
            // SAFETY: `entity_identifiers` contains the raw parts for a valid
            // `Vec<entity::Identifier>` of length `length`.
            unsafe {
                slice::from_raw_parts_mut::<'a, entity::Identifier>(entity_identifiers.0, length)
            }
            .iter()
            .copied(),
            // SAFETY: The components in `columns` are guaranteed to contain raw parts for valid
            // `Vec<C>`s of length `length` for each of the components identified by
            // `archetype_identifier`.
            unsafe {
                R::view(
                    columns.get_unchecked(1..),
                    length,
                    archetype_identifier,
                )
            },
        )
    }
}

impl<'a, I, P, R, V, Q> ContainsViews<'a, V, (NotContained, P), I, Q>
    for (EntityIdentifierMarker, R)
where
    R: CanonicalViews<'a, <R as ContainsViewsInner<'a, V, P, I>>::Canonical, P> + ContainsViewsInner<'a, V, P, I>,
    <<R as ContainsViewsInner<'a, V, P, I>>::Canonical as ViewsSeal<'a>>::Results:
        Reshape<<V as ViewsSeal<'a>>::Results, Q>,
    V: Views<'a>,
{
    type Canonical = <R as ContainsViewsInner<'a, V, P, I>>::Canonical;
    type CanonicalResults = <Self::Canonical as ViewsSeal<'a>>::Results;

    unsafe fn view<R_>(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> Self::CanonicalResults
    where
        R_: Registry,
    {
        // SAFETY: The components in `columns` are guaranteed to contain raw parts for valid
        // `Vec<C>`s of length `length` for each of the components identified by
        // `archetype_identifier`.
        unsafe { R::view(columns, length, archetype_identifier) }
    }
}

pub trait ContainsViewsInner<'a, V, P, I>
where
    V: Views<'a>,
{
    type Canonical: Views<'a>;
}

impl<'a> ContainsViewsInner<'a, view::Null, Null, Null> for registry::Null {
    type Canonical = view::Null;
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (&'a Contained, P), (I, IS)> for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as view::Get<'a, &'a C, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, &'a C, I>,
{
    type Canonical = (
        &'a C,
        <R as ContainsViewsInner<'a, <V as view::Get<'a, &'a C, I>>::Remainder, P, IS>>::Canonical,
    );
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (&'a mut Contained, P), (I, IS)> for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as view::Get<'a, &'a mut C, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, &'a mut C, I>,
{
    type Canonical = (
        &'a mut C,
        <R as ContainsViewsInner<'a, <V as view::Get<'a, &'a mut C, I>>::Remainder, P, IS>>::Canonical,
    );
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (Option<&'a Contained>, P), (I, IS)>
    for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as view::Get<'a, Option<&'a C>, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, Option<&'a C>, I>,
{
    type Canonical =
        (
            Option<&'a C>,
            <R as ContainsViewsInner<
                'a,
                <V as view::Get<'a, Option<&'a C>, I>>::Remainder,
                P,
                IS,
            >>::Canonical,
        );
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (Option<&'a mut Contained>, P), (I, IS)>
    for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as view::Get<'a, Option<&'a mut C>, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, Option<&'a mut C>, I>,
{
    type Canonical = (
        Option<&'a mut C>,
        <R as ContainsViewsInner<
            'a,
            <V as view::Get<'a, Option<&'a mut C>, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
}

impl<'a, I, IS, P, V, R> ContainsViewsInner<'a, V, (Contained, P), (I, IS)>
    for (EntityIdentifierMarker, R)
where
    R: ContainsViewsInner<'a, <V as view::Get<'a, entity::Identifier, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, entity::Identifier, I>,
{
    type Canonical = (
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
}

impl<'a, C, I, P, R, V> ContainsViewsInner<'a, V, (NotContained, P), I> for (C, R)
where
    R: ContainsViewsInner<'a, V, P, I>,
    V: Views<'a>,
{
    type Canonical = <R as ContainsViewsInner<'a, V, P, I>>::Canonical;
}

#[cfg(feature = "rayon")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub trait ContainsParViews<'a, V, P, I> {
    type Canonical: ParViews<'a>;
}

#[cfg(feature = "rayon")]
impl ContainsParViews<'_, view::Null, Null, Null> for registry::Null {
    type Canonical = view::Null;
}

#[cfg(feature = "rayon")]
impl<'a, C, I, IS, P, R, V> ContainsParViews<'a, V, (&'a Contained, P), (I, IS)> for (C, R)
where
    C: Component + Sync,
    R: ContainsParViews<'a, <V as view::Get<'a, &'a C, I>>::Remainder, P, IS>,
    V: view::Get<'a, &'a C, I>,
{
    type Canonical = (
        &'a C,
        <R as ContainsParViews<'a, <V as view::Get<'a, &'a C, I>>::Remainder, P, IS>>::Canonical,
    );
}

#[cfg(feature = "rayon")]
impl<'a, C, I, IS, P, R, V> ContainsParViews<'a, V, (&'a mut Contained, P), (I, IS)> for (C, R)
where
    C: Component + Send,
    R: ContainsParViews<'a, <V as view::Get<'a, &'a mut C, I>>::Remainder, P, IS>,
    V: view::Get<'a, &'a mut C, I>,
{
    type Canonical = (&'a mut C, <R as ContainsParViews<'a, <V as view::Get<'a, &'a mut C, I>>::Remainder, P, IS>>::Canonical);
}

#[cfg(feature = "rayon")]
impl<'a, C, I, IS, P, R, V> ContainsParViews<'a, V, (Option<&'a Contained>, P), (I, IS)> for (C, R)
where
    C: Component + Sync,
    R: ContainsParViews<'a, <V as view::Get<'a, Option<&'a C>, I>>::Remainder, P, IS>,
    V: view::Get<'a, Option<&'a C>, I>,
{
    type Canonical = (Option<&'a C>, <R as ContainsParViews<'a, <V as view::Get<'a, Option<&'a C>, I>>::Remainder, P, IS>>::Canonical);
}

#[cfg(feature = "rayon")]
impl<'a, C, I, IS, P, R, V> ContainsParViews<'a, V, (Option<&'a mut Contained>, P), (I, IS)>
    for (C, R)
where
    C: Component + Send,
    R: ContainsParViews<'a, <V as view::Get<'a, Option<&'a mut C>, I>>::Remainder, P, IS>,
    V: view::Get<'a, Option<&'a mut C>, I>,
{
    type Canonical =
        (
            Option<&'a mut C>,
            <R as ContainsParViews<
                'a,
                <V as view::Get<'a, Option<&'a mut C>, I>>::Remainder,
                P,
                IS,
            >>::Canonical,
        );
}

#[cfg(feature = "rayon")]
impl<'a, I, IS, P, V, R> ContainsParViews<'a, V, (Contained, P), (I, IS)>
    for (EntityIdentifierMarker, R)
where
    R: ContainsParViews<'a, <V as view::Get<'a, entity::Identifier, I>>::Remainder, P, IS>,
    V: view::Get<'a, entity::Identifier, I>,
{
    type Canonical = (
        entity::Identifier,
        <R as ContainsParViews<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
}

#[cfg(feature = "rayon")]
impl<'a, C, I, P, R, V> ContainsParViews<'a, V, (NotContained, P), I> for (C, R)
where
    R: ContainsParViews<'a, V, P, I>,
{
    type Canonical = <R as ContainsParViews<'a, V, P, I>>::Canonical;
}

#[cfg(test)]
mod component_tests {
    use super::ContainsComponent;
    use crate::registry;

    struct A;
    struct B;
    struct C;
    struct D;
    struct E;

    type Registry = registry!(A, B, C, D, E);

    #[test]
    fn contains_start() {
        assert_eq!(<Registry as ContainsComponent<A, _>>::INDEX, 4);
    }

    #[test]
    fn contains_middle() {
        assert_eq!(<Registry as ContainsComponent<C, _>>::INDEX, 2);
    }

    #[test]
    fn contains_end() {
        assert_eq!(<Registry as ContainsComponent<E, _>>::INDEX, 0);
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
