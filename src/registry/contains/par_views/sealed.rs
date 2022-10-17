use crate::{
    archetype,
    component::Component,
    entity,
    query::{
        result,
        view,
        view::{
            ParViews,
            ParViewsSeal,
        },
    },
    registry,
    registry::{
        contains::{
            Contained,
            EntityIdentifierMarker,
            NotContained,
            Null,
        },
        CanonicalParViews,
        Registry,
    },
};
use rayon::iter::{
    IntoParallelRefIterator,
    ParallelIterator,
};

#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub trait Sealed<'a, V, P, I, Q>: Registry
where
    V: ParViews<'a>,
{
    type Viewable: ContainsParViewsOuter<'a, V, P, I, Q>;
}

impl<'a, T, V, P, I, Q> Sealed<'a, V, P, I, Q> for T
where
    T: Registry,
    V: ParViews<'a>,
    (EntityIdentifierMarker, T): ContainsParViewsOuter<'a, V, P, I, Q>,
{
    type Viewable = (EntityIdentifierMarker, Self);
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
/// Indicates that all of the components viewed are contained in a registry.
///
/// This allows reordering the components viewed into a canonical form, as well as reordering the
/// results back to the originally requested form.
pub trait ContainsParViewsOuter<'a, V, P, I, Q>
where
    V: ParViews<'a>,
{
    /// The canonical form of the views `V`.
    type Canonical: ParViews<'a, ParResults = Self::CanonicalResults>;
    /// The canonical form of the results of the views `V`. Equivalent to
    /// `Self::Canonical::Results`.
    type CanonicalResults: result::Reshape<V::ParResults, Q>;

    /// # Safety
    ///
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`, ordered for the archetype identified by `archetype_identifier`.
    ///
    /// Additionally, `entity_identifiers` must contain the raw parts for a valid
    /// `Vec<entity::Identifier>` of length `length`.
    unsafe fn par_view<R>(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R>,
    ) -> Self::CanonicalResults
    where
        R: Registry;
}

impl<'a, I, IS, P, V, R, Q> ContainsParViewsOuter<'a, V, (Contained, P), (I, IS), Q>
    for (EntityIdentifierMarker, R)
where
    R: ContainsParViewsInner<'a, <V as view::Get<'a, entity::Identifier, I>>::Remainder, P, IS>
        + CanonicalParViews<
            'a,
            <R as ContainsParViewsInner<
                'a,
                <V as view::Get<'a, entity::Identifier, I>>::Remainder,
                P,
                IS,
            >>::Canonical,
            P,
        >,
    V: ParViews<'a> + view::Get<'a, entity::Identifier, I>,
    (
        rayon::iter::Cloned<rayon::slice::Iter<'a, entity::Identifier>>,
        <<R as ContainsParViewsInner<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical as ParViewsSeal<'a>>::ParResults,
    ): result::Reshape<<V as ParViewsSeal<'a>>::ParResults, Q>,
{
    type Canonical = (
        entity::Identifier,
        <R as ContainsParViewsInner<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
    type CanonicalResults = <Self::Canonical as ParViewsSeal<'a>>::ParResults;

    unsafe fn par_view<R_>(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> Self::CanonicalResults
    where
        R_: Registry,
    {
        (
            // SAFETY: `entity_identifiers` is guaranteed to contain the raw parts for a valid
            // `Vec<entity::Identifier>` of size `length`.
            unsafe {
                core::slice::from_raw_parts_mut::<'a, entity::Identifier>(
                    entity_identifiers.0,
                    length,
                )
            }
            .par_iter()
            .cloned(),
            // SAFETY: The components in `columns` are guaranteed to contain raw parts for valid
            // `Vec<C>`s of length `length` for each of the components identified by
            // `archetype_identifier`.
            unsafe { R::par_view(columns, length, archetype_identifier) },
        )
    }
}

impl<'a, I, P, R, V, Q> ContainsParViewsOuter<'a, V, (NotContained, P), I, Q>
    for (EntityIdentifierMarker, R)
where
    R: ContainsParViewsInner<'a, V, P, I>
        + CanonicalParViews<'a, <R as ContainsParViewsInner<'a, V, P, I>>::Canonical, P>,
    V: ParViews<'a>,
    <<R as ContainsParViewsInner<'a, V, P, I>>::Canonical as ParViewsSeal<'a>>::ParResults:
        result::Reshape<<V as ParViewsSeal<'a>>::ParResults, Q>,
{
    type Canonical = <R as ContainsParViewsInner<'a, V, P, I>>::Canonical;
    type CanonicalResults = <Self::Canonical as ParViewsSeal<'a>>::ParResults;

    unsafe fn par_view<R_>(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> Self::CanonicalResults
    where
        R_: Registry,
    {
        // SAFETY: The components in `columns` are guaranteed to contain raw parts for valid
        // `Vec<C>`s of length `length` for each of the components identified by
        // `archetype_identifier`.
        unsafe { R::par_view(columns, length, archetype_identifier) }
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
pub trait ContainsParViewsInner<'a, V, P, I> {
    type Canonical: ParViews<'a>;
}

impl ContainsParViewsInner<'_, view::Null, Null, Null> for registry::Null {
    type Canonical = view::Null;
}

impl<'a, C, I, IS, P, R, V> ContainsParViewsInner<'a, V, (&'a Contained, P), (I, IS)> for (C, R)
where
    C: Component + Sync,
    R: ContainsParViewsInner<'a, <V as view::Get<'a, &'a C, I>>::Remainder, P, IS>,
    V: view::Get<'a, &'a C, I>,
{
    type Canonical = (
        &'a C,
        <R as ContainsParViewsInner<'a, <V as view::Get<'a, &'a C, I>>::Remainder, P, IS>>::Canonical,
    );
}

impl<'a, C, I, IS, P, R, V> ContainsParViewsInner<'a, V, (&'a mut Contained, P), (I, IS)> for (C, R)
where
    C: Component + Send,
    R: ContainsParViewsInner<'a, <V as view::Get<'a, &'a mut C, I>>::Remainder, P, IS>,
    V: view::Get<'a, &'a mut C, I>,
{
    type Canonical =
        (
            &'a mut C,
            <R as ContainsParViewsInner<
                'a,
                <V as view::Get<'a, &'a mut C, I>>::Remainder,
                P,
                IS,
            >>::Canonical,
        );
}

impl<'a, C, I, IS, P, R, V> ContainsParViewsInner<'a, V, (Option<&'a Contained>, P), (I, IS)>
    for (C, R)
where
    C: Component + Sync,
    R: ContainsParViewsInner<'a, <V as view::Get<'a, Option<&'a C>, I>>::Remainder, P, IS>,
    V: view::Get<'a, Option<&'a C>, I>,
{
    type Canonical = (
        Option<&'a C>,
        <R as ContainsParViewsInner<
            'a,
            <V as view::Get<'a, Option<&'a C>, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
}

impl<'a, C, I, IS, P, R, V> ContainsParViewsInner<'a, V, (Option<&'a mut Contained>, P), (I, IS)>
    for (C, R)
where
    C: Component + Send,
    R: ContainsParViewsInner<'a, <V as view::Get<'a, Option<&'a mut C>, I>>::Remainder, P, IS>,
    V: view::Get<'a, Option<&'a mut C>, I>,
{
    type Canonical = (
        Option<&'a mut C>,
        <R as ContainsParViewsInner<
            'a,
            <V as view::Get<'a, Option<&'a mut C>, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
}

impl<'a, C, I, P, R, V> ContainsParViewsInner<'a, V, (NotContained, P), I> for (C, R)
where
    R: ContainsParViewsInner<'a, V, P, I>,
{
    type Canonical = <R as ContainsParViewsInner<'a, V, P, I>>::Canonical;
}
