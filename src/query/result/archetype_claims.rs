//! Iterator over dynamic claims of a query on its affected archetypes.

use crate::{
    archetype,
    archetypes,
    query::{
        view,
        view::Claims,
    },
    registry,
    registry::{
        contains::{
            filter::Sealed as ContainsFilterSealed,
            views::Sealed as ContainsViewsSealed,
        },
        ContainsFilter,
        ContainsQuery,
        ContainsViews,
    },
};
use core::marker::PhantomData;

/// Iterator over dynamic claims of a query on its affected archetypes.
///
/// This iterator returns key-value pairs of archetype identifiers and the list of claimed
/// components for the given query on that archetype.
pub struct ArchetypeClaims<
    'a,
    Registry,
    Views,
    QueryFilter,
    Filter,
    EntryViews,
    QueryIndices,
    FilterIndices,
    EntryViewsIndices,
> where
    Registry: registry::Registry,
{
    archetypes_iter: archetypes::IterMut<'a, Registry>,

    views: PhantomData<Views>,
    query_filter: PhantomData<QueryFilter>,
    filter: PhantomData<Filter>,
    entry_views: PhantomData<EntryViews>,
    query_indices: PhantomData<QueryIndices>,
    filter_indices: PhantomData<FilterIndices>,
    entry_views_indices: PhantomData<EntryViewsIndices>,
}

impl<
        'a,
        Registry,
        Views,
        QueryFilter,
        Filter,
        EntryViews,
        QueryIndices,
        FilterIndices,
        EntryViewsIndices,
    >
    ArchetypeClaims<
        'a,
        Registry,
        Views,
        QueryFilter,
        Filter,
        EntryViews,
        QueryIndices,
        FilterIndices,
        EntryViewsIndices,
    >
where
    Registry: registry::Registry,
{
    /// Returns a new `ArchetypeClaims` iterator.
    ///
    /// # Safety
    /// The `archetype::IdentifierRef`s over which this iterator iterates must not outlive the
    /// `Archetypes` to which they belong.
    pub(crate) unsafe fn new(archetypes_iter: archetypes::IterMut<'a, Registry>) -> Self {
        Self {
            archetypes_iter,

            views: PhantomData,
            query_filter: PhantomData,
            filter: PhantomData,
            entry_views: PhantomData,
            query_indices: PhantomData,
            filter_indices: PhantomData,
            entry_views_indices: PhantomData,
        }
    }
}

impl<
        'a,
        Registry,
        Views,
        QueryFilter,
        Filter,
        EntryViews,
        QueryIndices,
        FilterIndices,
        EntryViewsIndices,
    > Iterator
    for ArchetypeClaims<
        'a,
        Registry,
        Views,
        QueryFilter,
        Filter,
        EntryViews,
        QueryIndices,
        FilterIndices,
        EntryViewsIndices,
    >
where
    Views: view::Views<'a>,
    EntryViews: view::Views<'a>,
    Registry: ContainsFilter<Filter, FilterIndices>
        + ContainsQuery<'a, QueryFilter, Views, QueryIndices>
        + ContainsViews<'a, EntryViews, EntryViewsIndices>,
{
    type Item = (archetype::IdentifierRef<Registry>, Registry::Claims);

    fn next(&mut self) -> Option<Self::Item> {
        self.archetypes_iter
            .find(|archetype| {
                // SAFETY: The `R` on which `filter()` is called is the same `R` over which the
                // identifier is generic over. Additionally, the identifier reference created here
                // will not outlive `archetype`.
                unsafe {
                    <Registry as ContainsFilterSealed<Filter, FilterIndices>>::filter(
                        archetype.identifier(),
                    )
                }
            })
            .map(|archetype| {
                (
                    // SAFETY: The `IdentifierRef` created here is guaranteed to outlive
                    // `archetype`, so long as the safety contract at construction is upheld.
                    unsafe { archetype.identifier() },
                    unsafe {
                        <Registry as ContainsViewsSealed<
                            'a,
                            Views,
                            (
                                Registry::ViewsContainments,
                                Registry::ViewsIndices,
                                Registry::ViewsCanonicalContainments,
                            ),
                        >>::claims()
                        .merge_unchecked(&<Registry as ContainsViewsSealed<
                            'a,
                            EntryViews,
                            EntryViewsIndices,
                        >>::claims())
                    },
                )
            })
    }
}
