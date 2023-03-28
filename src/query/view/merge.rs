//! Merge two non-conflicting canonical views into a single one.
//!
//! Note that the output views do **not** respect optional views on components. These views are
//! only intended to be used in scheduling, and probably shouldn't be used to actually view
//! components. Please proceed with caution.

use crate::{
    entity,
    hlist::define_null,
    query::view,
    registry::{
        self,
        contains::EntityIdentifierMarker,
    },
};

define_null!();

pub enum Neither {}
pub enum Left {}
pub enum Right {}
pub enum Both {}

pub trait Merge<Views, OtherViews, Containments> {
    type Merged;
}

// Base case.
impl Merge<view::Null, view::Null, Null> for registry::Null {
    type Merged = view::Null;
}

// Neither.
impl<Component, Registry, Views, OtherViews, Containments>
    Merge<Views, OtherViews, (Neither, Containments)> for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = <Registry as Merge<Views, OtherViews, Containments>>::Merged;
}

// Left.
impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<(&'a Component, Views), OtherViews, (Left, Containments)> for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<(&'a mut Component, Views), OtherViews, (Left, Containments)> for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a mut Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<(Option<&'a Component>, Views), OtherViews, (Left, Containments)>
    for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<(Option<&'a mut Component>, Views), OtherViews, (Left, Containments)>
    for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a mut Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<Registry, Views, OtherViews, Containments>
    Merge<(entity::Identifier, Views), OtherViews, (Left, Containments)>
    for (EntityIdentifierMarker, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        entity::Identifier,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

// Right.
impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<Views, (&'a Component, OtherViews), (Right, Containments)> for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<Views, (&'a mut Component, OtherViews), (Right, Containments)> for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a mut Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<Views, (Option<&'a Component>, OtherViews), (Right, Containments)>
    for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<Views, (Option<&'a mut Component>, OtherViews), (Right, Containments)>
    for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a mut Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<Registry, Views, OtherViews, Containments>
    Merge<Views, (entity::Identifier, OtherViews), (Right, Containments)>
    for (EntityIdentifierMarker, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        entity::Identifier,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

// Both
impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<(&'a Component, Views), (&'a Component, OtherViews), (Both, Containments)>
    for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<(&'a Component, Views), (Option<&'a Component>, OtherViews), (Both, Containments)>
    for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<(Option<&'a Component>, Views), (&'a Component, OtherViews), (Both, Containments)>
    for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<'a, Component, Registry, Views, OtherViews, Containments>
    Merge<(Option<&'a Component>, Views), (Option<&'a Component>, OtherViews), (Both, Containments)>
    for (Component, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        &'a Component,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

impl<Registry, Views, OtherViews, Containments>
    Merge<(entity::Identifier, Views), (entity::Identifier, OtherViews), (Both, Containments)>
    for (EntityIdentifierMarker, Registry)
where
    Registry: Merge<Views, OtherViews, Containments>,
{
    type Merged = (
        entity::Identifier,
        <Registry as Merge<Views, OtherViews, Containments>>::Merged,
    );
}

#[cfg(test)]
mod tests {
    use super::Merge;
    use crate::{
        entity,
        query::Views,
        registry::contains::EntityIdentifierMarker,
        Registry,
    };
    use core::any::TypeId;

    struct A;
    struct B;
    struct C;

    type Registry = Registry!(A, B, C);

    #[test]
    fn empty() {
        assert_eq!(
            TypeId::of::<Views!()>(),
            TypeId::of::<<Registry as Merge<Views!(), Views!(), _>>::Merged>()
        );
    }

    #[test]
    fn left_empty_right_immutable() {
        assert_eq!(
            TypeId::of::<Views!(&A)>(),
            TypeId::of::<<Registry as Merge<Views!(), Views!(&A), _>>::Merged>()
        );
    }

    #[test]
    fn left_empty_right_mutable() {
        assert_eq!(
            TypeId::of::<Views!(&mut A)>(),
            TypeId::of::<<Registry as Merge<Views!(), Views!(&mut A), _>>::Merged>()
        );
    }

    #[test]
    fn left_empty_right_optional_immutable() {
        assert_eq!(
            TypeId::of::<Views!(&A)>(),
            TypeId::of::<<Registry as Merge<Views!(), Views!(Option<&A>), _>>::Merged>()
        );
    }

    #[test]
    fn left_empty_right_optional_mutable() {
        assert_eq!(
            TypeId::of::<Views!(&mut A)>(),
            TypeId::of::<<Registry as Merge<Views!(), Views!(Option<&mut A>), _>>::Merged>()
        );
    }

    #[test]
    fn right_empty_left_immutable() {
        assert_eq!(
            TypeId::of::<Views!(&A)>(),
            TypeId::of::<<Registry as Merge<Views!(&A), Views!(), _>>::Merged>()
        );
    }

    #[test]
    fn right_empty_left_mutable() {
        assert_eq!(
            TypeId::of::<Views!(&mut A)>(),
            TypeId::of::<<Registry as Merge<Views!(&mut A), Views!(), _>>::Merged>()
        );
    }

    #[test]
    fn right_empty_left_optional_immutable() {
        assert_eq!(
            TypeId::of::<Views!(&A)>(),
            TypeId::of::<<Registry as Merge<Views!(Option<&A>), Views!(), _>>::Merged>()
        );
    }

    #[test]
    fn right_empty_left_optional_mutable() {
        assert_eq!(
            TypeId::of::<Views!(&mut A)>(),
            TypeId::of::<<Registry as Merge<Views!(Option<&mut A>), Views!(), _>>::Merged>()
        );
    }

    #[test]
    fn both_views_contain_immutable_reference() {
        assert_eq!(
            TypeId::of::<Views!(&A)>(),
            TypeId::of::<<Registry as Merge<Views!(&A), Views!(&A), _>>::Merged>()
        );
    }

    #[test]
    fn left_optional() {
        assert_eq!(
            TypeId::of::<Views!(&A)>(),
            TypeId::of::<<Registry as Merge<Views!(Option<&A>), Views!(&A), _>>::Merged>()
        );
    }

    #[test]
    fn right_optional() {
        assert_eq!(
            TypeId::of::<Views!(&A)>(),
            TypeId::of::<<Registry as Merge<Views!(&A), Views!(Option<&A>), _>>::Merged>()
        );
    }

    #[test]
    fn different_views() {
        assert_eq!(
            TypeId::of::<Views!(&mut A, &B, &mut C)>(),
            TypeId::of::<<Registry as Merge<Views!(&mut A, &B), Views!(&B, &mut C), _>>::Merged>()
        );
    }

    #[test]
    fn entity_identifier_left() {
        assert_eq!(
            TypeId::of::<Views!(entity::Identifier)>(),
            TypeId::of::<
                <(EntityIdentifierMarker, Registry) as Merge<
                    Views!(entity::Identifier),
                    Views!(),
                    _,
                >>::Merged,
            >()
        );
    }

    #[test]
    fn entity_identifier_right() {
        assert_eq!(
            TypeId::of::<Views!(entity::Identifier)>(),
            TypeId::of::<
                <(EntityIdentifierMarker, Registry) as Merge<
                    Views!(),
                    Views!(entity::Identifier),
                    _,
                >>::Merged,
            >()
        );
    }

    #[test]
    fn entity_identifier_both() {
        assert_eq!(
            TypeId::of::<Views!(entity::Identifier)>(),
            TypeId::of::<
                <(EntityIdentifierMarker, Registry) as Merge<
                    Views!(entity::Identifier),
                    Views!(entity::Identifier),
                    _,
                >>::Merged,
            >()
        );
    }
}
