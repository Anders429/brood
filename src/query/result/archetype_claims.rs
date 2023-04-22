//! Iterator over dynamic claims of a query on its affected archetypes.

use crate::{
    archetype,
    archetypes,
    query::{
        filter::And,
        view,
    },
    registry,
    registry::{
        contains::filter::Sealed as ContainsFilterSealed,
        ContainsQuery,
    },
};
use core::marker::PhantomData;

/// Iterator over dynamic claims of a query on its affected archetypes.
///
/// This iterator returns key-value pairs of archetype identifiers and the list of claimed
/// components for the given query on that archetype.
pub struct ArchetypeClaims<'a, Registry, Filter, Views, Indices>
where
    Registry: registry::Registry,
{
    archetypes_iter: archetypes::IterMut<'a, Registry>,

    filter: PhantomData<Filter>,
    view: PhantomData<Views>,
    indices: PhantomData<Indices>,
}

impl<'a, Registry, Filter, Views, Indices> ArchetypeClaims<'a, Registry, Filter, Views, Indices>
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

            filter: PhantomData,
            view: PhantomData,
            indices: PhantomData,
        }
    }
}

impl<'a, Registry, Filter, Views, Indices> Iterator
    for ArchetypeClaims<'a, Registry, Filter, Views, Indices>
where
    Views: view::Views<'a>,
    Registry: ContainsQuery<'a, Filter, Views, Indices>,
{
    type Item = (archetype::IdentifierRef<Registry>, Registry::Claims);

    fn next(&mut self) -> Option<Self::Item> {
        self.archetypes_iter
            .find(|archetype| {
                // SAFETY: The `R` on which `filter()` is called is the same `R` over which the
                // identifier is generic over. Additionally, the identifier reference created here
                // will not outlive `archetype`.
                unsafe {
                    <Registry as ContainsFilterSealed<
                        And<Views, Filter>,
                        And<Registry::ViewsFilterIndices, Registry::FilterIndices>,
                    >>::filter(archetype.identifier())
                }
            })
            .map(|archetype| {
                (
                    // SAFETY: The `IdentifierRef` created here is guaranteed to outlive
                    // `archetype`, so long as the safety contract at construction is upheld.
                    unsafe { archetype.identifier() },
                    Registry::claims(),
                )
            })
    }
}
