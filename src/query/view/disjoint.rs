//! Indicates that two sets of `Views` do not conflict with each other.
//!
//! Basic algorithm used here:
//! - Find the inverse of one set of views.
//! - Find the canonical form of the other set of views.
//! - Ensure the components contained in the canonical form are all contained in the inverse.

use crate::{
    entity,
    hlist::define_null,
    query::view,
    registry,
    registry::ContainsViews,
};

define_null!();

/// Indicates that one set of [`Views`] does not conflict with another set of `Views`.
///
/// Two conflicting views on a single component occur when one of the two views tries to access the
/// component mutably and the other tries to access the component at all. In other words, the views
/// violating Rust's borrowing rules is what creates a conflict.
///
/// [`Views`]: trait@crate::query::view::Views
pub trait Disjoint<
    OtherViews,
    Registry,
    Indices,
    InverseIndices,
    OppositeIndices,
    OppositeInverseIndices,
>:
    Sealed<OtherViews, Registry, Indices, InverseIndices, OppositeIndices, OppositeInverseIndices>
{
}

impl<
        Views,
        OtherViews,
        Registry,
        Indices,
        InverseIndices,
        OppositeIndices,
        OppositeInverseIndices,
    >
    Disjoint<OtherViews, Registry, Indices, InverseIndices, OppositeIndices, OppositeInverseIndices>
    for Views
where
    Views: Sealed<
        OtherViews,
        Registry,
        Indices,
        InverseIndices,
        OppositeIndices,
        OppositeInverseIndices,
    >,
{
}

pub trait Sealed<
    OtherViews,
    Registry,
    Indices,
    InverseIndices,
    OppositeIndices,
    OppositeInverseIndices,
>
{
}

impl<
        'a,
        Views,
        OtherViews,
        Registry,
        Indices,
        InverseIndices,
        OppositeIndices,
        OppositeInverseIndices,
    > Sealed<OtherViews, Registry, Indices, InverseIndices, OppositeIndices, OppositeInverseIndices>
    for Views
where
    OtherViews: view::Views<'a> + MutableInverse<Registry, InverseIndices>,
    OtherViews::Result: ContainsViews<'a, Views, Indices>,
    Views: view::Views<'a> + MutableInverse<Registry, OppositeInverseIndices>,
    Views::Result: ContainsViews<'a, OtherViews, OppositeIndices>,
{
}

pub trait MutableInverse<Registry, Indices> {
    type Result;
}

impl<Registry> MutableInverse<Registry, Null> for view::Null {
    type Result = Registry;
}

impl<Component, Views, Registry, Indices> MutableInverse<Registry, Indices> for (&Component, Views)
where
    Views: MutableInverse<Registry, Indices>,
{
    type Result = <Views as MutableInverse<Registry, Indices>>::Result;
}

impl<Component, Views, Registry, Indices> MutableInverse<Registry, Indices>
    for (Option<&Component>, Views)
where
    Views: MutableInverse<Registry, Indices>,
{
    type Result = <Views as MutableInverse<Registry, Indices>>::Result;
}

impl<Views, Registry, Indices> MutableInverse<Registry, Indices> for (entity::Identifier, Views)
where
    Views: MutableInverse<Registry, Indices>,
{
    type Result = <Views as MutableInverse<Registry, Indices>>::Result;
}

impl<Component, Views, Registry, Index, Indices> MutableInverse<Registry, (Index, Indices)>
    for (&mut Component, Views)
where
    Registry: registry::Get<Component, Index>,
    Views: MutableInverse<<Registry as registry::Get<Component, Index>>::Remainder, Indices>,
{
    type Result = <Views as MutableInverse<
        <Registry as registry::Get<Component, Index>>::Remainder,
        Indices,
    >>::Result;
}

impl<Component, Views, Registry, Index, Indices> MutableInverse<Registry, (Index, Indices)>
    for (Option<&mut Component>, Views)
where
    Registry: registry::Get<Component, Index>,
    Views: MutableInverse<<Registry as registry::Get<Component, Index>>::Remainder, Indices>,
{
    type Result = <Views as MutableInverse<
        <Registry as registry::Get<Component, Index>>::Remainder,
        Indices,
    >>::Result;
}

#[cfg(test)]
mod tests {
    use super::Disjoint;
    use crate::{
        entity,
        query::Views,
        Registry,
    };

    fn is_disjoint<
        Views,
        OtherViews,
        Registry,
        Indices,
        InverseIndices,
        OppositeIndices,
        OppositeInverseIndices,
    >()
    where
        Views: Disjoint<
            OtherViews,
            Registry,
            Indices,
            InverseIndices,
            OppositeIndices,
            OppositeInverseIndices,
        >,
    {
    }

    // Define components.
    struct A;
    struct B;
    struct C;

    type Registry = Registry!(A, B, C);

    #[test]
    fn empty() {
        is_disjoint::<Views!(), Views!(), Registry!(), _, _, _, _>();
    }

    #[test]
    fn empty_first_views() {
        is_disjoint::<Views!(), Views!(&A, &mut B, Option<&C>), Registry, _, _, _, _>();
    }

    #[test]
    fn empty_second_views() {
        is_disjoint::<Views!(&A, &mut B, Option<&C>), Views!(), Registry, _, _, _, _>();
    }

    #[test]
    fn shared_immutable_views() {
        is_disjoint::<Views!(&A, &B, &C), Views!(&A, &B, &C), Registry, _, _, _, _>();
    }

    #[test]
    fn shared_immutable_optional_views() {
        is_disjoint::<
            Views!(Option<&A>, Option<&B>, Option<&C>),
            Views!(Option<&A>, Option<&B>, Option<&C>),
            Registry,
            _,
            _,
            _,
            _,
        >();
    }

    #[test]
    fn disjoint_mutable_views() {
        is_disjoint::<
            Views!(&mut A, &mut C),
            Views!(&mut B, entity::Identifier),
            Registry,
            _,
            _,
            _,
            _,
        >();
    }

    #[test]
    fn disjoint_mutable_optional_views() {
        is_disjoint::<
            Views!(Option<&mut A>, Option<&mut C>),
            Views!(Option<&mut B>, entity::Identifier),
            Registry,
            _,
            _,
            _,
            _,
        >();
    }

    #[test]
    fn entity_identifier() {
        is_disjoint::<Views!(entity::Identifier), Views!(entity::Identifier), Registry, _, _, _, _>(
        );
    }
}
