//! Iterator over dynamic claims of a query on its affected archetypes.

use crate::{
    archetype,
    archetypes,
    query::{
        filter::And,
        view::Views,
    },
    registry::{
        contains::filter::Sealed as ContainsFilterSealed,
        ContainsQuery,
        Registry,
    },
};
use core::marker::PhantomData;

/// Iterator over dynamic claims of a query on its affected archetypes.
///
/// This iterator returns key-value pairs of archetype identifiers and the list of claimed
/// components for the given query on that archetype.
pub struct ArchetypeClaims<'a, R, F, FI, V, VI, P, I, Q>
where
    R: Registry,
{
    archetypes_iter: archetypes::IterMut<'a, R>,

    filter: PhantomData<F>,
    filter_indices: PhantomData<FI>,
    view: PhantomData<V>,
    view_filter_indices: PhantomData<VI>,
    view_containments: PhantomData<P>,
    view_indices: PhantomData<I>,
    reshape_indices: PhantomData<Q>,
}

impl<'a, R, F, FI, V, VI, P, I, Q> ArchetypeClaims<'a, R, F, FI, V, VI, P, I, Q>
where
    R: Registry,
{
    /// Returns a new `ArchetypeClaims` iterator.
    ///
    /// # Safety
    /// The `archetype::IdentifierRef`s over which this iterator iterates must not outlive the
    /// `Archetypes` to which they belong.
    pub(crate) unsafe fn new(archetypes_iter: archetypes::IterMut<'a, R>) -> Self {
        Self {
            archetypes_iter,

            filter: PhantomData,
            filter_indices: PhantomData,
            view: PhantomData,
            view_filter_indices: PhantomData,
            view_containments: PhantomData,
            view_indices: PhantomData,
            reshape_indices: PhantomData,
        }
    }
}

impl<'a, R, F, FI, V, VI, P, I, Q> Iterator for ArchetypeClaims<'a, R, F, FI, V, VI, P, I, Q>
where
    V: Views<'a>,
    R: ContainsQuery<'a, F, FI, V, VI, P, I, Q>,
{
    type Item = (archetype::IdentifierRef<R>, R::Claims);

    fn next(&mut self) -> Option<Self::Item> {
        self.archetypes_iter
            .find(|archetype| {
                // SAFETY: The `R` on which `filter()` is called is the same `R` over which the
                // identifier is generic over. Additionally, the identifier reference created here
                // will not outlive `archetype`.
                unsafe {
                    <R as ContainsFilterSealed<And<V, F>, And<VI, FI>>>::filter(
                        archetype.identifier(),
                    )
                }
            })
            .map(|archetype| {
                (
                    // SAFETY: The `IdentifierRef` created here is guaranteed to outlive
                    // `archetype`, so long as the safety contract at construction is upheld.
                    unsafe { archetype.identifier() },
                    R::claims(),
                )
            })
    }
}
