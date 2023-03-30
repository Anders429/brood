//! Indicates that one `Views` is a "subset" of another `Views`.
//!
//! Here, a `View` being contained in another `Views` means that the component viewed is also
//! viewed in `Views`. Additionally, a mutable `View` must also be mutably viewed in `Views`, while
//! the same restriction does not apply to immutable views.
//!
//! The basic algorithm here is to:
//! - "Get" each view within the subview inside the superview.
//! - The Get trait only is implemented for the subset relationship for each component.
//! - End case is having null::Views in subset.

use crate::{
    entity,
    hlist::define_null,
    query::view,
};

define_null!();

mod index {
    pub enum Index {}
}

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
/// [`Views`]: trait@crate::query::view::Views
pub trait SubSet<Views, Indices>: Sealed<Views, Indices> {}

impl<SubViews, Views, Indices> SubSet<Views, Indices> for SubViews where
    SubViews: Sealed<Views, Indices>
{
}

pub trait Sealed<Views, Indices> {}

impl<Views> Sealed<Views, Null> for view::Null {}

impl<SubView, SubViews, Views, Index, Indices> Sealed<Views, (Index, Indices)>
    for (SubView, SubViews)
where
    Views: Get<SubView, Index>,
    SubViews: Sealed<Views::Remainder, Indices>,
{
}

/// This `Get` implementation follows the "subset" rules defined for views. If the subview is
/// contained within the superview, it can be found with this `Get` implementation.
pub trait Get<View, Index> {
    type Remainder;
}

impl<'a, Component, Views> Get<&'a Component, index::Index> for (&'a Component, Views) {
    type Remainder = Views;
}

impl<'a, Component, Views> Get<&'a Component, index::Index> for (&'a mut Component, Views) {
    type Remainder = Views;
}

impl<'a, Component, Views> Get<&'a Component, index::Index> for (Option<&'a Component>, Views) {
    type Remainder = Views;
}

impl<'a, Component, Views> Get<&'a Component, index::Index> for (Option<&'a mut Component>, Views) {
    type Remainder = Views;
}

impl<'a, Component, Views> Get<&'a mut Component, index::Index> for (&'a mut Component, Views) {
    type Remainder = Views;
}

impl<'a, Component, Views> Get<&'a mut Component, index::Index>
    for (Option<&'a mut Component>, Views)
{
    type Remainder = Views;
}

impl<'a, Component, Views> Get<Option<&'a Component>, index::Index> for (&'a Component, Views) {
    type Remainder = Views;
}

impl<'a, Component, Views> Get<Option<&'a Component>, index::Index> for (&'a mut Component, Views) {
    type Remainder = Views;
}

impl<'a, Component, Views> Get<Option<&'a Component>, index::Index>
    for (Option<&'a Component>, Views)
{
    type Remainder = Views;
}

impl<'a, Component, Views> Get<Option<&'a Component>, index::Index>
    for (Option<&'a mut Component>, Views)
{
    type Remainder = Views;
}

impl<'a, Component, Views> Get<Option<&'a mut Component>, index::Index>
    for (&'a mut Component, Views)
{
    type Remainder = Views;
}

impl<'a, Component, Views> Get<Option<&'a mut Component>, index::Index>
    for (Option<&'a mut Component>, Views)
{
    type Remainder = Views;
}

impl<Views> Get<entity::Identifier, index::Index> for (entity::Identifier, Views) {
    type Remainder = Views;
}

impl<View, OtherView, Views, Index> Get<View, (Index,)> for (OtherView, Views)
where
    Views: Get<View, Index>,
{
    type Remainder = (OtherView, <Views as Get<View, Index>>::Remainder);
}

#[cfg(test)]
mod tests {
    use super::SubSet;
    use crate::{
        entity,
        query::Views,
    };

    // Components.
    struct A;
    struct B;
    struct C;

    fn is_subset<ViewsSubSet, Views, Indices>()
    where
        ViewsSubSet: SubSet<Views, Indices>,
    {
    }

    #[test]
    fn empty() {
        is_subset::<Views!(), Views!(), _>();
    }

    #[test]
    fn empty_nonempty_registry() {
        is_subset::<Views!(), Views!(), _>();
    }

    #[test]
    fn empty_nonempty_views() {
        is_subset::<Views!(), Views!(&A, &mut B), _>();
    }

    #[test]
    fn immutable_subset_of_immutable() {
        is_subset::<Views!(&A), Views!(&A, &mut B), _>();
    }

    #[test]
    fn immutable_subset_of_mutable() {
        is_subset::<Views!(&B), Views!(&A, &mut B), _>();
    }

    #[test]
    fn mutable_subset_of_mutable() {
        is_subset::<Views!(&mut B), Views!(&A, &mut B), _>();
    }

    #[test]
    fn optional_immutable_subset_of_immutable() {
        is_subset::<Views!(Option<&A>), Views!(&A, &mut B), _>();
    }

    #[test]
    fn optional_immutable_subset_of_mutable() {
        is_subset::<Views!(Option<&B>), Views!(&A, &mut B), _>();
    }

    #[test]
    fn optional_mutable_subset_of_mutable() {
        is_subset::<Views!(Option<&mut B>), Views!(&A, &mut B), _>();
    }

    #[test]
    fn entity_identifier() {
        is_subset::<Views!(entity::Identifier), Views!(&A, entity::Identifier, &C), _>();
    }

    #[test]
    fn multiple_subset_views() {
        is_subset::<Views!(&A, Option<&mut B>, &mut C), Views!(&A, &mut B, &mut C), _>();
    }

    #[test]
    fn canonical_order() {
        is_subset::<
            Views!(&mut B, &C, Option<&A>, entity::Identifier),
            Views!(&mut C, &A, entity::Identifier, &mut B),
            _,
        >();
    }
}
