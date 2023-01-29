use crate::{
    archetype,
    archetypes,
    query::{
        filter::{
            And,
            Filter,
        },
        view::Views,
    },
    registry::{
        contains::filter::Sealed as ContainsFilterSealed,
        ContainsQuery,
        Registry,
    },
};
use core::marker::PhantomData;

pub struct ArchetypeIdentifiers<'a, R, F, FI, V, VI, P, I, Q>
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

impl<'a, R, F, FI, V, VI, P, I, Q> ArchetypeIdentifiers<'a, R, F, FI, V, VI, P, I, Q>
where
    R: Registry,
{
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

impl<'a, R, F, FI, V, VI, P, I, Q> Iterator for ArchetypeIdentifiers<'a, R, F, FI, V, VI, P, I, Q>
where
    F: Filter,
    V: Views<'a> + Filter,
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
            .map(|archetype| unsafe { (archetype.identifier(), R::claims()) })
    }
}
