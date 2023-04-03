//! Denotes that a filter can be applied upon a set of views.

use crate::{
    archetype,
    component,
    entity,
    query::{
        filter::{
            And,
            Has,
            None,
            Not,
            Or,
        },
        view,
    },
    registry,
};

mod index {
    pub enum Index {}
}

/// Denotes that a filter can be applied upon views.
///
/// This means that any components viewed can also be filtered on, treating the views as a
/// sub-registry.
pub trait ContainsFilter<'a, Filter, Index>: Sealed<'a, Filter, Index> {}

impl<'a, Views, Filter, Index> ContainsFilter<'a, Filter, Index> for Views where
    Views: Sealed<'a, Filter, Index>
{
}

pub trait Sealed<'a, Filter, Index>: view::Views<'a> + Sized {
    /// # Safety
    /// `indices` must contain valid indices into `Registry` (and therefore valid indices into
    /// `identifier`).
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry;
}

impl<'a, Component, Views> Sealed<'a, Has<Component>, index::Index> for (&'a Component, Views)
where
    Component: component::Component,
    Views: view::Views<'a>,
    Self: view::Views<'a, Indices = (usize, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        let index = indices.0;
        // SAFETY: `index` is guaranteed to be a valid index into the `Registry`.
        unsafe { identifier.get_unchecked(index) }
    }
}

impl<'a, Component, Views> Sealed<'a, Has<Component>, index::Index> for (&'a mut Component, Views)
where
    Component: component::Component,
    Views: view::Views<'a>,
    Self: view::Views<'a, Indices = (usize, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        let index = indices.0;
        // SAFETY: `index` is guaranteed to be a valid index into the `Registry`.
        unsafe { identifier.get_unchecked(index) }
    }
}

impl<'a, Component, Views> Sealed<'a, Has<Component>, index::Index>
    for (Option<&'a Component>, Views)
where
    Component: component::Component,
    Views: view::Views<'a>,
    Self: view::Views<'a, Indices = (usize, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        let index = indices.0;
        // SAFETY: `index` is guaranteed to be a valid index into the `Registry`.
        unsafe { identifier.get_unchecked(index) }
    }
}

impl<'a, Component, Views> Sealed<'a, Has<Component>, index::Index>
    for (Option<&'a mut Component>, Views)
where
    Component: component::Component,
    Views: view::Views<'a>,
    Self: view::Views<'a, Indices = (usize, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        let index = indices.0;
        // SAFETY: `index` is guaranteed to be a valid index into the `Registry`.
        unsafe { identifier.get_unchecked(index) }
    }
}

impl<'a, Component, View, Views, Index> Sealed<'a, Has<Component>, (Index,)> for (View, Views)
where
    Component: component::Component,
    View: view::View<'a>,
    Views: Sealed<'a, Has<Component>, Index>,
    Self: view::Views<'a, Indices = (View::Index, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        // SAFETY: `indices.1` is guaranteed to contain valid indices into `Registry`.
        unsafe { Views::filter(&indices.1, identifier) }
    }
}

impl<'a, FilterA, FilterB, Views, IndexA, IndexB>
    Sealed<'a, And<FilterA, FilterB>, And<IndexA, IndexB>> for Views
where
    Views: Sealed<'a, FilterA, IndexA> + Sealed<'a, FilterB, IndexB>,
    Self: view::Views<'a>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        // SAFETY: `indices` is guaranteed to contain valid indices into `Registry`.
        unsafe {
            <Views as Sealed<'a, FilterA, IndexA>>::filter(indices, identifier)
                && <Views as Sealed<'a, FilterB, IndexB>>::filter(indices, identifier)
        }
    }
}

impl<'a, FilterA, FilterB, Views, IndexA, IndexB>
    Sealed<'a, Or<FilterA, FilterB>, Or<IndexA, IndexB>> for Views
where
    Views: Sealed<'a, FilterA, IndexA> + Sealed<'a, FilterB, IndexB>,
    Self: view::Views<'a>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        // SAFETY: `indices` is guaranteed to contain valid indices into `Registry`.
        unsafe {
            <Views as Sealed<'a, FilterA, IndexA>>::filter(indices, identifier)
                || <Views as Sealed<'a, FilterB, IndexB>>::filter(indices, identifier)
        }
    }
}

impl<'a, Filter, Views, Index> Sealed<'a, Not<Filter>, Index> for Views
where
    Views: Sealed<'a, Filter, Index>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        // SAFETY: `indices` is guaranteed to contain valid indices into `Registry`.
        !unsafe { Views::filter(indices, identifier) }
    }
}

impl<'a, Views> Sealed<'a, None, index::Index> for Views
where
    Self: view::Views<'a>,
{
    unsafe fn filter<Registry>(
        _indices: &Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        true
    }
}

impl<'a, Component, Views> Sealed<'a, &'a Component, index::Index> for (&'a Component, Views)
where
    Views: view::Views<'a>,
    Self: view::Views<'a, Indices = (usize, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        let index = indices.0;
        // SAFETY: `index` is guaranteed to be a valid index into the `Registry`.
        unsafe { identifier.get_unchecked(index) }
    }
}

impl<'a, Component, Views> Sealed<'a, Option<&'a Component>, index::Index> for Views
where
    Self: view::Views<'a>,
{
    unsafe fn filter<Registry>(
        _indices: &Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        true
    }
}

impl<'a, Component, Views> Sealed<'a, &'a Component, index::Index> for (&'a mut Component, Views)
where
    Views: view::Views<'a>,
    Self: view::Views<'a, Indices = (usize, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        let index = indices.0;
        // SAFETY: `index` is guaranteed to be a valid index into the `Registry`.
        unsafe { identifier.get_unchecked(index) }
    }
}

impl<'a, Component, Views> Sealed<'a, &'a mut Component, index::Index>
    for (&'a mut Component, Views)
where
    Views: view::Views<'a>,
    Self: view::Views<'a, Indices = (usize, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        let index = indices.0;
        // SAFETY: `index` is guaranteed to be a valid index into the `Registry`.
        unsafe { identifier.get_unchecked(index) }
    }
}

impl<'a, Component, Views> Sealed<'a, Option<&'a mut Component>, index::Index> for Views
where
    Self: view::Views<'a>,
{
    unsafe fn filter<Registry>(
        _indices: &Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        true
    }
}

impl<'a, Component, Views> Sealed<'a, &'a Component, index::Index>
    for (Option<&'a Component>, Views)
where
    Views: view::Views<'a>,
    Self: view::Views<'a, Indices = (usize, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        let index = indices.0;
        // SAFETY: `index` is guaranteed to be a valid index into the `Registry`.
        unsafe { identifier.get_unchecked(index) }
    }
}

impl<'a, Component, Views> Sealed<'a, &'a Component, index::Index>
    for (Option<&'a mut Component>, Views)
where
    Views: view::Views<'a>,
    Self: view::Views<'a, Indices = (usize, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        let index = indices.0;
        // SAFETY: `index` is guaranteed to be a valid index into the `Registry`.
        unsafe { identifier.get_unchecked(index) }
    }
}

impl<'a, Component, Views> Sealed<'a, &'a mut Component, index::Index>
    for (Option<&'a mut Component>, Views)
where
    Views: view::Views<'a>,
    Self: view::Views<'a, Indices = (usize, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        let index = indices.0;
        // SAFETY: `index` is guaranteed to be a valid index into the `Registry`.
        unsafe { identifier.get_unchecked(index) }
    }
}

impl<'a, Component, View, Views, Index> Sealed<'a, &'a Component, (Index,)> for (View, Views)
where
    Component: component::Component,
    View: view::View<'a>,
    Views: Sealed<'a, &'a Component, Index>,
    Self: view::Views<'a, Indices = (View::Index, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        // SAFETY: `indices.1` is guaranteed to contain valid indices into `Registry`.
        unsafe { Views::filter(&indices.1, identifier) }
    }
}

impl<'a, Component, View, Views, Index> Sealed<'a, &'a mut Component, (Index,)> for (View, Views)
where
    Component: component::Component,
    View: view::View<'a>,
    Views: Sealed<'a, &'a mut Component, Index>,
    Self: view::Views<'a, Indices = (View::Index, Views::Indices)>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        // SAFETY: `indices.1` is guaranteed to contain valid indices into `Registry`.
        unsafe { Views::filter(&indices.1, identifier) }
    }
}

impl<'a, Views> Sealed<'a, entity::Identifier, index::Index> for Views
where
    Self: view::Views<'a>,
{
    unsafe fn filter<Registry>(
        _indices: &Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        true
    }
}

impl<'a, Views> Sealed<'a, view::Null, index::Index> for Views
where
    Self: view::Views<'a>,
{
    unsafe fn filter<Registry>(
        _indices: &Self::Indices,
        _identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        true
    }
}

impl<'a, Views, Filter, Filters, Index, Indices> Sealed<'a, (Filter, Filters), (Index, Indices)>
    for Views
where
    Self: view::Views<'a>,
    Views: Sealed<'a, Filter, Index> + Sealed<'a, Filters, Indices>,
{
    unsafe fn filter<Registry>(
        indices: &Self::Indices,
        identifier: archetype::IdentifierRef<Registry>,
    ) -> bool
    where
        Registry: registry::Registry,
    {
        // SAFETY: `indices` is guaranteed to contain valid indices into `Registry`.
        unsafe {
            <Views as Sealed<'a, Filter, Index>>::filter(indices, identifier)
                && <Views as Sealed<'a, Filters, Indices>>::filter(indices, identifier)
        }
    }
}
