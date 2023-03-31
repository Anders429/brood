use crate::{
    archetype,
    component::Component,
    entity,
    query::{
        result,
        view,
        view::{
            Reshape,
            Views,
            ViewsSealed,
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
        CanonicalViews,
        Registry,
    },
};
use core::{
    iter,
    slice,
};

pub trait Sealed<'a, V, P, I, Q>: Registry
where
    V: Views<'a>,
{
    type Viewable: ContainsViewsOuter<'a, V, P, I, Q>;

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims;
}

impl<'a, T, V, P, I, Q> Sealed<'a, V, P, I, Q> for T
where
    T: Registry,
    V: Views<'a>,
    (EntityIdentifierMarker, T): ContainsViewsOuter<'a, V, P, I, Q, Registry = T>,
{
    type Viewable = (EntityIdentifierMarker, Self);

    /// Return the dynamic claims over the components borrowed by the `Views`.
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims {
        Self::Viewable::claims()
    }
}

/// Indicates that all of the components viewed are contained in a registry.
///
/// This allows reordering the components viewed into a canonical form, as well as reordering the
/// results back to the originally requested form.
pub trait ContainsViewsOuter<'a, V, P, I, Q>
where
    V: Views<'a>,
{
    /// The underlying `Registry`.
    ///
    /// The type on which this trait is implemented is a registry combined with an entity
    /// identifier. This associated type allows access directly to that registry.
    type Registry: Registry;
    /// The canonical form of the views `V`.
    type Canonical: Views<'a>
        + ViewsSealed<'a, Results = Self::CanonicalResults>
        + view::Reshape<'a, V, Q>;
    /// The canonical form of the results of the views `V`. Equivalent to
    /// `Self::Canonical::Results`.
    type CanonicalResults: result::Reshape<V::Results, Q>;

    /// # Safety
    ///
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`, ordered for the archetype identified by `archetype_identifier`.
    ///
    /// Additionally, `entity_identifiers` must contain the raw parts for a valid
    /// `Vec<entity::Identifier>` of length `length`.
    unsafe fn view<R>(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R>,
    ) -> Self::CanonicalResults
    where
        R: Registry;

    /// # Safety
    ///
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`, ordered for the archetype identified by `archetype_identifier`.
    ///
    /// Additionally, `entity_identifiers` must contain the raw parts for a valid
    /// `Vec<entity::Identifier>` of length `length`.
    ///
    /// Finally, `index` must be a less than `length`.
    unsafe fn view_one<R>(
        index: usize,
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R>,
    ) -> Self::Canonical
    where
        R: Registry;

    /// Return the dynamic claims over the components borrowed by the `Views`.
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> <Self::Registry as registry::sealed::Claims>::Claims;

    fn indices() -> V::Indices;
}

impl<'a, I, IS, P, V, R, Q> ContainsViewsOuter<'a, V, (Contained, P), (I, IS), Q>
    for (EntityIdentifierMarker, R)
where
    R: CanonicalViews<
            'a,
            <R as ContainsViewsInner<
                'a,
                <V as view::Get<'a, entity::Identifier, I>>::Remainder,
                P,
                IS,
            >>::Canonical,
            P,
        > + ContainsViewsInner<'a, <V as view::Get<'a, entity::Identifier, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, entity::Identifier, I>,
    <(
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    ) as ViewsSealed<'a>>::Results: result::Reshape<<V as ViewsSealed<'a>>::Results, Q>,
    (
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    ): view::Reshape<'a, V, Q>
        + ViewsSealed<
            'a,
            Results = (
                iter::Copied<slice::Iter<'a, entity::Identifier>>,
                <<R as ContainsViewsInner<
                    'a,
                    <V as view::Get<'a, entity::Identifier, I>>::Remainder,
                    P,
                    IS,
                >>::Canonical as ViewsSealed<'a>>::Results,
            ),
            Indices = (
                view::Null,
                <<R as ContainsViewsInner<
                    'a,
                    <V as view::Get<'a, entity::Identifier, I>>::Remainder,
                    P,
                    IS,
                >>::Canonical as ViewsSealed<'a>>::Indices,
            ),
        >,
{
    type Registry = R;
    type Canonical = (
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
    type CanonicalResults = <Self::Canonical as ViewsSealed<'a>>::Results;

    unsafe fn view<R_>(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> Self::CanonicalResults
    where
        R_: Registry,
    {
        (
            // SAFETY: `entity_identifiers` contains the raw parts for a valid
            // `Vec<entity::Identifier>` of length `length`.
            unsafe {
                slice::from_raw_parts_mut::<'a, entity::Identifier>(entity_identifiers.0, length)
            }
            .iter()
            .copied(),
            // SAFETY: The components in `columns` are guaranteed to contain raw parts for valid
            // `Vec<C>`s of length `length` for each of the components identified by
            // `archetype_identifier`.
            unsafe { R::view(columns, length, archetype_identifier) },
        )
    }

    unsafe fn view_one<R_>(
        index: usize,
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> Self::Canonical
    where
        R_: Registry,
    {
        (
            // SAFETY: `entity_identifiers` is guaranteed to contain the raw parts for a valid
            // `Vec<entity::Identifier>` of size `length`. Consequentially, `index` is guaranteed
            // to be a valid index into the `Vec<entity::Identifier>`.
            *unsafe {
                slice::from_raw_parts_mut::<'a, entity::Identifier>(entity_identifiers.0, length)
                    .get_unchecked(index)
            },
            // SAFETY: The components in `columns` are guaranteed to contain raw parts for valid
            // `Vec<C>`s of length `length` for each of the components identified by
            // `archetype_identifier`. `index` is guaranteed to be less than `length`.
            unsafe { R::view_one(index, columns, length, archetype_identifier) },
        )
    }

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> <Self::Registry as registry::sealed::Claims>::Claims {
        R::claims()
    }

    fn indices() -> V::Indices {
        let canonical_indices = (
            view::Null,
            <R as CanonicalViews<
                'a,
                <R as ContainsViewsInner<
                    'a,
                    <V as view::Get<'a, entity::Identifier, I>>::Remainder,
                    P,
                    IS,
                >>::Canonical,
                P,
            >>::indices::<R>(),
        );
        <(
            entity::Identifier,
            <R as ContainsViewsInner<
                'a,
                <V as view::Get<'a, entity::Identifier, I>>::Remainder,
                P,
                IS,
            >>::Canonical,
        )>::reshape_indices(canonical_indices)
    }
}

impl<'a, I, P, R, V, Q> ContainsViewsOuter<'a, V, (NotContained, P), I, Q>
    for (EntityIdentifierMarker, R)
where
    R: CanonicalViews<'a, <R as ContainsViewsInner<'a, V, P, I>>::Canonical, P>
        + ContainsViewsInner<'a, V, P, I>,
    <<R as ContainsViewsInner<'a, V, P, I>>::Canonical as ViewsSealed<'a>>::Results:
        result::Reshape<<V as ViewsSealed<'a>>::Results, Q>,
    <R as ContainsViewsInner<'a, V, P, I>>::Canonical: view::Reshape<'a, V, Q>,
    V: Views<'a>,
{
    type Registry = R;
    type Canonical = <R as ContainsViewsInner<'a, V, P, I>>::Canonical;
    type CanonicalResults = <Self::Canonical as ViewsSealed<'a>>::Results;

    unsafe fn view<R_>(
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
        unsafe { R::view(columns, length, archetype_identifier) }
    }

    unsafe fn view_one<R_>(
        index: usize,
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> Self::Canonical
    where
        R_: Registry,
    {
        // SAFETY: The components in `columns` are guaranteed to contain raw parts for valid
        // `Vec<C>`s of length `length` for each of the components identified by
        // `archetype_identifier`. `index` is guaranteed to be less than `length`.
        unsafe { R::view_one(index, columns, length, archetype_identifier) }
    }

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> <Self::Registry as registry::sealed::Claims>::Claims {
        R::claims()
    }

    fn indices() -> V::Indices {
        let canonical_indices = R::indices::<R>();
        Self::Canonical::reshape_indices(canonical_indices)
    }
}

pub trait ContainsViewsInner<'a, V, P, I>
where
    V: Views<'a>,
{
    type Canonical: Views<'a>;
}

impl<'a> ContainsViewsInner<'a, view::Null, Null, Null> for registry::Null {
    type Canonical = view::Null;
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (&'a Contained, P), (I, IS)> for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as view::Get<'a, &'a C, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, &'a C, I>,
{
    type Canonical = (
        &'a C,
        <R as ContainsViewsInner<'a, <V as view::Get<'a, &'a C, I>>::Remainder, P, IS>>::Canonical,
    );
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (&'a mut Contained, P), (I, IS)> for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as view::Get<'a, &'a mut C, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, &'a mut C, I>,
{
    type Canonical = (
        &'a mut C,
        <R as ContainsViewsInner<'a, <V as view::Get<'a, &'a mut C, I>>::Remainder, P, IS>>::Canonical,
    );
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (Option<&'a Contained>, P), (I, IS)>
    for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as view::Get<'a, Option<&'a C>, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, Option<&'a C>, I>,
{
    type Canonical =
        (
            Option<&'a C>,
            <R as ContainsViewsInner<
                'a,
                <V as view::Get<'a, Option<&'a C>, I>>::Remainder,
                P,
                IS,
            >>::Canonical,
        );
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (Option<&'a mut Contained>, P), (I, IS)>
    for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as view::Get<'a, Option<&'a mut C>, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, Option<&'a mut C>, I>,
{
    type Canonical = (
        Option<&'a mut C>,
        <R as ContainsViewsInner<
            'a,
            <V as view::Get<'a, Option<&'a mut C>, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
}

impl<'a, I, IS, P, V, R> ContainsViewsInner<'a, V, (Contained, P), (I, IS)>
    for (EntityIdentifierMarker, R)
where
    R: ContainsViewsInner<'a, <V as view::Get<'a, entity::Identifier, I>>::Remainder, P, IS>,
    V: Views<'a> + view::Get<'a, entity::Identifier, I>,
{
    type Canonical = (
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as view::Get<'a, entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
}

impl<'a, C, I, P, R, V> ContainsViewsInner<'a, V, (NotContained, P), I> for (C, R)
where
    R: ContainsViewsInner<'a, V, P, I>,
    V: Views<'a>,
{
    type Canonical = <R as ContainsViewsInner<'a, V, P, I>>::Canonical;
}
