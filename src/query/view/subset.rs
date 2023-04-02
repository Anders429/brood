//! Indicates that one `Views` is a "subset" of another `Views`.
//!
//! Here, a `View` being contained in another `Views` means that the component viewed is also
//! viewed in `Views`. Additionally, a mutable `View` must also be mutably viewed in `Views`, while
//! the same restriction does not apply to immutable views.
//!
//! The basic algorithm here is to:
//! - "Get" each view within the subview inside the superview.
//! - The Get trait only is implemented for the subset relationship for each component.
//! - End case is having `null::Views` in subset.

use crate::{
    archetype,
    entity,
    hlist::define_null,
    query::view,
    registry,
};
use core::{
    mem,
    mem::MaybeUninit,
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
pub trait SubSet<'a, Views, Indices>: Sealed<'a, Views, Indices>
where
    Views: view::Views<'a>,
{
}

impl<'a, SubViews, Views, Indices> SubSet<'a, Views, Indices> for SubViews
where
    SubViews: Sealed<'a, Views, Indices>,
    Views: view::Views<'a>,
{
}

pub trait Sealed<'a, Views, Indices>
where
    Views: view::Views<'a>,
{
    /// # Safety
    /// `views` must be a view on the archetype identified by `identifier`. `indices` must
    /// correspond to the `Registry` components.
    ///
    /// The current subview must have already been filtered for this archetype.
    unsafe fn view<Registry>(
        views: Views::MaybeUninit,
        indices: Views::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> Self
    where
        Registry: registry::Registry;
}

impl<'a, Views> Sealed<'a, Views, Null> for view::Null
where
    Views: view::Views<'a>,
{
    unsafe fn view<Registry>(
        _views: Views::MaybeUninit,
        _indices: Views::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> view::Null
    where
        Registry: registry::Registry,
    {
        view::Null
    }
}

impl<'a, SubView, SubViews, Views, Index, Indices> Sealed<'a, Views, (Index, Indices)>
    for (SubView, SubViews)
where
    Views: Get<'a, SubView, Index> + view::Views<'a>,
    <Views as Get<'a, SubView, Index>>::Remainder: view::Views<'a>,
    SubViews: Sealed<'a, Views::Remainder, Indices>,
{
    unsafe fn view<Registry>(
        views: Views::MaybeUninit,
        indices: Views::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> (SubView, SubViews)
    where
        Registry: registry::Registry,
    {
        let (target, (remainder, remainder_indices)) =
            unsafe { Views::view(views, indices, identifier) };

        (target, unsafe {
            SubViews::view(remainder, remainder_indices, identifier)
        })
    }
}

/// This `Get` implementation follows the "subset" rules defined for views. If the subview is
/// contained within the superview, it can be found with this `Get` implementation.
pub trait Get<'a, View, Index>: view::Views<'a> + Sized {
    type Remainder: view::Views<'a>;

    /// # Safety
    /// `views` must be a view on the archetype identified by `identifier`. `indices` must
    /// correspond to the `Registry` components.
    ///
    /// The current subview must have already been filtered for this archetype.
    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        View,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry;
}

impl<'a, Component, Views> Get<'a, &'a Component, index::Index> for (&'a Component, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (MaybeUninit<&'a Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        &'a Component,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (
            // SAFETY: This view has already been filtered for the archetype, so this component is
            // guaranteed to exist.
            unsafe { views.0.assume_init() },
            (views.1, indices.1),
        )
    }
}

impl<'a, Component, Views> Get<'a, &'a Component, index::Index> for (&'a mut Component, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (MaybeUninit<&'a mut Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        &'a Component,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (
            // SAFETY: This view has already been filtered for the archetype, so this component is
            // guaranteed to exist.
            unsafe { views.0.assume_init() },
            (views.1, indices.1),
        )
    }
}

impl<'a, Component, Views> Get<'a, &'a Component, index::Index> for (Option<&'a Component>, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (Option<&'a Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        &'a Component,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (
            // SAFETY: This view has already been filtered for the archetype, so this component is
            // guaranteed to exist.
            unsafe { views.0.unwrap_unchecked() },
            (views.1, indices.1),
        )
    }
}

impl<'a, Component, Views> Get<'a, &'a Component, index::Index>
    for (Option<&'a mut Component>, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (Option<&'a mut Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        &'a Component,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (
            // SAFETY: This view has already been filtered for the archetype, so this component is
            // guaranteed to exist.
            unsafe { views.0.unwrap_unchecked() },
            (views.1, indices.1),
        )
    }
}

impl<'a, Component, Views> Get<'a, &'a mut Component, index::Index> for (&'a mut Component, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (MaybeUninit<&'a mut Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        &'a mut Component,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (
            // SAFETY: This view has already been filtered for the archetype, so this component is
            // guaranteed to exist.
            unsafe { views.0.assume_init() },
            (views.1, indices.1),
        )
    }
}

impl<'a, Component, Views> Get<'a, &'a mut Component, index::Index>
    for (Option<&'a mut Component>, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (Option<&'a mut Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        &'a mut Component,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (
            // SAFETY: This view has already been filtered for the archetype, so this component is
            // guaranteed to exist.
            unsafe { views.0.unwrap_unchecked() },
            (views.1, indices.1),
        )
    }
}

impl<'a, Component, Views> Get<'a, Option<&'a Component>, index::Index> for (&'a Component, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (MaybeUninit<&'a Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        Option<&'a Component>,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (
            // SAFETY: `indices.0` is guaranteed to be a valid index in `identifier`. If it is set,
            // the component possibly viewed by `views.0` is guaranteed to exist.
            unsafe {
                if identifier.get_unchecked(indices.0) {
                    Some(views.0.assume_init())
                } else {
                    None
                }
            },
            (views.1, indices.1),
        )
    }
}

impl<'a, Component, Views> Get<'a, Option<&'a Component>, index::Index>
    for (&'a mut Component, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (MaybeUninit<&'a mut Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        Option<&'a Component>,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (
            // SAFETY: `indices.0` is guaranteed to be a valid index in `identifier`. If it is set,
            // the component possibly viewed by `views.0` is guaranteed to exist.
            unsafe {
                if identifier.get_unchecked(indices.0) {
                    Some(views.0.assume_init())
                } else {
                    None
                }
            },
            (views.1, indices.1),
        )
    }
}

impl<'a, Component, Views> Get<'a, Option<&'a Component>, index::Index>
    for (Option<&'a Component>, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (Option<&'a Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        Option<&'a Component>,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (views.0, (views.1, indices.1))
    }
}

impl<'a, Component, Views> Get<'a, Option<&'a Component>, index::Index>
    for (Option<&'a mut Component>, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (Option<&'a mut Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        Option<&'a Component>,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (
            // SAFETY: Transmuting Option<&mut T> as Option<&T> is a safe conversion.
            unsafe { mem::transmute(views.0) },
            (views.1, indices.1),
        )
    }
}

impl<'a, Component, Views> Get<'a, Option<&'a mut Component>, index::Index>
    for (&'a mut Component, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (MaybeUninit<&'a mut Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        Option<&'a mut Component>,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (
            // SAFETY: `indices.0` is guaranteed to be a valid index in `identifier`. If it is set,
            // the component possibly viewed by `views.0` is guaranteed to exist.
            unsafe {
                if identifier.get_unchecked(indices.0) {
                    Some(views.0.assume_init())
                } else {
                    None
                }
            },
            (views.1, indices.1),
        )
    }
}

impl<'a, Component, Views> Get<'a, Option<&'a mut Component>, index::Index>
    for (Option<&'a mut Component>, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (Option<&'a mut Component>, Views::MaybeUninit),
        Indices = (usize, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        Option<&'a mut Component>,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (views.0, (views.1, indices.1))
    }
}

impl<'a, Views> Get<'a, entity::Identifier, index::Index> for (entity::Identifier, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (entity::Identifier, Views::MaybeUninit),
        Indices = (view::Null, Views::Indices),
    >,
    Views: view::Views<'a>,
{
    type Remainder = Views;

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        entity::Identifier,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        (views.0, (views.1, indices.1))
    }
}

impl<'a, View, OtherView, Views, Index> Get<'a, View, (Index,)> for (OtherView, Views)
where
    Self: view::Views<
        'a,
        MaybeUninit = (OtherView::MaybeUninit, Views::MaybeUninit),
        Indices = (OtherView::Index, Views::Indices),
    >,
    Views: Get<'a, View, Index> + view::Views<'a>,
    OtherView: view::View<'a>,
{
    type Remainder = (OtherView, <Views as Get<'a, View, Index>>::Remainder);

    unsafe fn view<Registry>(
        views: Self::MaybeUninit,
        indices: Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> (
        View,
        (
            <Self::Remainder as view::ViewsSealed<'a>>::MaybeUninit,
            <Self::Remainder as view::ViewsSealed<'a>>::Indices,
        ),
    )
    where
        Registry: registry::Registry,
    {
        // SAFETY: The invariants are guaranteed to be upheld by the safety contract of this
        // function.
        let (target, (remainder, remainder_indices)) =
            unsafe { Views::view(views.1, indices.1, identifier) };
        (
            target,
            ((views.0, remainder), (indices.0, remainder_indices)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SubSet;
    use crate::{
        entity,
        query::{
            view,
            Views,
        },
    };

    // Components.
    struct A;
    struct B;
    struct C;

    fn is_subset<'a, ViewsSubSet, Views, Indices>()
    where
        ViewsSubSet: SubSet<'a, Views, Indices>,
        Views: view::Views<'a>,
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
