//! Indicates that one `Views` is a "subset" of another `Views`.
//!
//! Here, a `View` being contained in another `Views` means that the component viewed is also
//! viewed in `Views`. Additionally, a mutable `View` must also be mutably viewed in `Views`, while
//! the same restriction does not apply to immutable views.
//!
//! The basic algorithm here is to:
//! - Obtain the canonical version of both `Views`.
//! - Proceed through each view one by one.
//! - If they do not match, proceed to the next view in the superset, but keep the subset view the
//! same.
//! - If both `Views` get to `Null`, then it is a subset. Otherwise, it is not, and the trait is
//! not implemented.

use crate::{
    entity,
    hlist::define_null,
    query::view,
    registry::contains::views::{
        ContainsViewsOuter,
        Sealed as ContainsViewsSealed,
    },
};

define_null!();

pub enum Contained {}
pub enum NotContained {}

/// Defines one set of [`Views`] as a subset of another.
///
/// A `Views` is considered to be a subset of another `Views` if:
/// - Every component viewed by the subset is also viewed by the superset
/// - Every component viewed immutably (such as by `&T`) in the superset is also viewed immutably
/// in the subset
///
/// A component viewed mutably in the superset can be viewed either mutably or immutably in the
/// subset.
///
/// Components viewed must also be contained in the `Registry`.
///
/// This `trait` is automatically implemented for any pair of views that meet the above conditions.
/// It is never needed for a user to manually implement this `trait`.
///
/// [`Views`]: crate::query::views::Views
pub trait SubSet<
    Registry,
    Views,
    Containments,
    Indices,
    ReshapeIndices,
    ViewsContainments,
    ViewsIndices,
    ViewsReshapeIndices,
    CanonicalContainments,
>:
    Sealed<
    Registry,
    Views,
    Containments,
    Indices,
    ReshapeIndices,
    ViewsContainments,
    ViewsIndices,
    ViewsReshapeIndices,
    CanonicalContainments,
>
{
}

impl<
        T,
        Registry,
        Views,
        Containments,
        Indices,
        ReshapeIndices,
        ViewsContainments,
        ViewsIndices,
        ViewsReshapeIndices,
        CanonicalContainments,
    >
    SubSet<
        Registry,
        Views,
        Containments,
        Indices,
        ReshapeIndices,
        ViewsContainments,
        ViewsIndices,
        ViewsReshapeIndices,
        CanonicalContainments,
    > for T
where
    T: Sealed<
        Registry,
        Views,
        Containments,
        Indices,
        ReshapeIndices,
        ViewsContainments,
        ViewsIndices,
        ViewsReshapeIndices,
        CanonicalContainments,
    >,
{
}

pub trait Sealed<
    Registry,
    Views,
    Containments,
    Indices,
    ReshapeIndices,
    ViewsContainments,
    ViewsIndices,
    ViewsReshapeIndices,
    CanonicalContainments,
>
{
}

impl<'a, SubSetViews, Registry, Views, Containments, Indices, ReshapeIndices, ViewsContainments, ViewsIndices, ViewsReshapeIndices, CanonicalContainments> Sealed<Registry, Views, Containments, Indices, ReshapeIndices, ViewsContainments, ViewsIndices, ViewsReshapeIndices, CanonicalContainments> for SubSetViews
where
    SubSetViews: view::Views<'a>,
    Views: view::Views<'a>,
    Registry: ContainsViewsSealed<'a, SubSetViews, Containments, Indices, ReshapeIndices> + ContainsViewsSealed<'a, Views, ViewsContainments, ViewsIndices, ViewsReshapeIndices>,
    <<Registry as ContainsViewsSealed<'a, SubSetViews, Containments, Indices, ReshapeIndices>>::Viewable as ContainsViewsOuter<'a, SubSetViews, Containments, Indices, ReshapeIndices>>::Canonical: CanonicalSubSet<<<Registry as ContainsViewsSealed<'a, Views, ViewsContainments, ViewsIndices, ViewsReshapeIndices>>::Viewable as ContainsViewsOuter<'a, Views, ViewsContainments, ViewsIndices, ViewsReshapeIndices>>::Canonical, CanonicalContainments>,
{

}

/// This trait is implemented only on canonical forms of views.
///
/// This is used within the `Sealed` trait after converting the views to their canonical form. This
/// ensures the views contain components in the same order, which allows for a very
/// straight-forward implementation here.
///
/// This is where the actual subset detection is implemented. The other traits are just helper
/// traits to get the views in the form needed (canonical form) for this trait to do the heavy
/// lifting.
pub trait CanonicalSubSet<Views, Containments> {}

impl CanonicalSubSet<view::Null, Null> for view::Null {}

impl<SubSetViews, View, Views, Containments>
    CanonicalSubSet<(View, Views), (NotContained, Containments)> for SubSetViews
where
    SubSetViews: CanonicalSubSet<Views, Containments>,
{
}

impl<'a, SubSetViews, View, Views, Containments>
    CanonicalSubSet<(&'a View, Views), (Contained, Containments)> for (&'a View, SubSetViews)
where
    SubSetViews: CanonicalSubSet<Views, Containments>,
{
}

impl<'a, SubSetViews, View, Views, Containments>
    CanonicalSubSet<(&'a mut View, Views), (Contained, Containments)> for (&'a View, SubSetViews)
where
    SubSetViews: CanonicalSubSet<Views, Containments>,
{
}

impl<'a, SubSetViews, View, Views, Containments>
    CanonicalSubSet<(&'a mut View, Views), (Contained, Containments)>
    for (&'a mut View, SubSetViews)
where
    SubSetViews: CanonicalSubSet<Views, Containments>,
{
}

impl<'a, SubSetViews, View, Views, Containments>
    CanonicalSubSet<(&'a View, Views), (Contained, Containments)>
    for (Option<&'a View>, SubSetViews)
where
    SubSetViews: CanonicalSubSet<Views, Containments>,
{
}

impl<'a, SubSetViews, View, Views, Containments>
    CanonicalSubSet<(&'a mut View, Views), (Contained, Containments)>
    for (Option<&'a View>, SubSetViews)
where
    SubSetViews: CanonicalSubSet<Views, Containments>,
{
}

impl<'a, SubSetViews, View, Views, Containments>
    CanonicalSubSet<(&'a mut View, Views), (Contained, Containments)>
    for (Option<&'a mut View>, SubSetViews)
where
    SubSetViews: CanonicalSubSet<Views, Containments>,
{
}

impl<'a, SubSetViews, Views, Containments>
    CanonicalSubSet<(entity::Identifier, Views), (Contained, Containments)>
    for (entity::Identifier, SubSetViews)
where
    SubSetViews: CanonicalSubSet<Views, Containments>,
{
}

#[cfg(test)]
mod tests {
    use super::SubSet;
    use crate::{
        entity,
        query::Views,
        Registry,
    };

    // Components.
    struct A;
    struct B;
    struct C;

    type Registry = Registry!(A, B, C);

    fn is_subset<
        ViewsSubSet,
        Views,
        Registry,
        Containments,
        Indices,
        ReshapeIndices,
        ViewsContainments,
        ViewsIndices,
        ViewsReshapeIndices,
        CanonicalContainments,
    >()
    where
        ViewsSubSet: SubSet<
            Registry,
            Views,
            Containments,
            Indices,
            ReshapeIndices,
            ViewsContainments,
            ViewsIndices,
            ViewsReshapeIndices,
            CanonicalContainments,
        >,
    {
    }

    #[test]
    fn empty() {
        is_subset::<Views!(), Views!(), Registry!(), _, _, _, _, _, _, _>();
    }

    #[test]
    fn empty_nonempty_registry() {
        is_subset::<Views!(), Views!(), Registry, _, _, _, _, _, _, _>();
    }

    #[test]
    fn empty_nonempty_views() {
        is_subset::<Views!(), Views!(&A, &mut B), Registry, _, _, _, _, _, _, _>();
    }

    #[test]
    fn immutable_subset_of_immutable() {
        is_subset::<Views!(&A), Views!(&A, &mut B), Registry, _, _, _, _, _, _, _>();
    }

    #[test]
    fn immutable_subset_of_mutable() {
        is_subset::<Views!(&B), Views!(&A, &mut B), Registry, _, _, _, _, _, _, _>();
    }

    #[test]
    fn mutable_subset_of_mutable() {
        is_subset::<Views!(&mut B), Views!(&A, &mut B), Registry, _, _, _, _, _, _, _>();
    }

    #[test]
    fn optional_immutable_subset_of_immutable() {
        is_subset::<Views!(Option<&A>), Views!(&A, &mut B), Registry, _, _, _, _, _, _, _>();
    }

    #[test]
    fn optional_immutable_subset_of_mutable() {
        is_subset::<Views!(Option<&B>), Views!(&A, &mut B), Registry, _, _, _, _, _, _, _>();
    }

    #[test]
    fn optional_mutable_subset_of_mutable() {
        is_subset::<Views!(Option<&mut B>), Views!(&A, &mut B), Registry, _, _, _, _, _, _, _>();
    }

    #[test]
    fn entity_identifier() {
        is_subset::<
            Views!(entity::Identifier),
            Views!(&A, entity::Identifier, &C),
            Registry,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
        >();
    }

    #[test]
    fn multiple_subset_views() {
        is_subset::<
            Views!(&A, Option<&mut B>, &mut C),
            Views!(&A, &mut B, &mut C),
            Registry,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
        >();
    }

    #[test]
    fn canonical_order() {
        is_subset::<
            Views!(&mut B, &C, Option<&A>, entity::Identifier),
            Views!(&mut C, &A, entity::Identifier, &mut B),
            Registry,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
        >();
    }
}
