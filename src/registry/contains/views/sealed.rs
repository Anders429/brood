use crate::{
    archetype,
    component::Component,
    entity,
    hlist::{
        Get,
        Reshape,
    },
    query::{
        view,
        view::{
            Reshape as _,
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

pub trait Sealed<'a, Views, Indices>: Registry
where
    Views: view::Views<'a>,
{
    type Containments;
    type Indices;
    type ReshapeIndices;
    type Viewable: ContainsViewsOuter<
        'a,
        Views,
        Self::Containments,
        Self::Indices,
        Self::ReshapeIndices,
    >;

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims;
}

impl<'a, Registry, Views, Containments, Indices, ReshapeIndices>
    Sealed<'a, Views, (Containments, Indices, ReshapeIndices)> for Registry
where
    Registry: registry::Registry,
    Views: view::Views<'a>,
    (EntityIdentifierMarker, Registry):
        ContainsViewsOuter<'a, Views, Containments, Indices, ReshapeIndices, Registry = Registry>,
{
    type Containments = Containments;
    type Indices = Indices;
    type ReshapeIndices = ReshapeIndices;
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
    type CanonicalResults: Reshape<V::Results, Q, iter::Take<iter::Repeat<view::Null>>>;

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

    unsafe fn view_one_maybe_uninit<R>(
        index: usize,
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R>,
    ) -> <Self::Canonical as ViewsSealed<'a>>::MaybeUninit
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
                <V as Get<entity::Identifier, I>>::Remainder,
                P,
                IS,
            >>::Canonical,
            P,
        > + ContainsViewsInner<'a, <V as Get<entity::Identifier, I>>::Remainder, P, IS>,
    V: Views<'a> + Get<entity::Identifier, I>,
    V::Remainder: Views<'a>,
    <(
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as Get<entity::Identifier, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    ) as ViewsSealed<'a>>::Results: Reshape<<V as ViewsSealed<'a>>::Results, Q, iter::Take<iter::Repeat<view::Null>>>,
    (
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as Get<entity::Identifier, I>>::Remainder,
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
                    <V as Get<entity::Identifier, I>>::Remainder,
                    P,
                    IS,
                >>::Canonical as ViewsSealed<'a>>::Results,
            ),
            Indices = (
                view::Null,
                <<R as ContainsViewsInner<
                    'a,
                    <V as Get<entity::Identifier, I>>::Remainder,
                    P,
                    IS,
                >>::Canonical as ViewsSealed<'a>>::Indices,
            ),
            MaybeUninit = (
                entity::Identifier,
                <<R as ContainsViewsInner<
                    'a,
                    <V as Get<entity::Identifier, I>>::Remainder,
                    P,
                    IS,
                >>::Canonical as ViewsSealed<'a>>::MaybeUninit,
            ),
        >,
{
    type Registry = R;
    type Canonical = (
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as Get<entity::Identifier, I>>::Remainder,
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

    unsafe fn view_one_maybe_uninit<R_>(
        index: usize,
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <Self::Canonical as ViewsSealed<'a>>::MaybeUninit
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
            unsafe { R::view_one_maybe_uninit(index, columns, length, archetype_identifier) },
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
                    <V as Get<entity::Identifier, I>>::Remainder,
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
                <V as Get<entity::Identifier, I>>::Remainder,
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
        Reshape<<V as ViewsSealed<'a>>::Results, Q, iter::Take<iter::Repeat<view::Null>>>,
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

    unsafe fn view_one_maybe_uninit<R_>(
        index: usize,
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <Self::Canonical as ViewsSealed<'a>>::MaybeUninit
    where
        R_: Registry,
    {
        // SAFETY: The safety contract of this function applies to this function call.
        unsafe { R::view_one_maybe_uninit(index, columns, length, archetype_identifier) }
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
    R: ContainsViewsInner<'a, <V as Get<&'a C, I>>::Remainder, P, IS>,
    V: Views<'a> + Get<&'a C, I>,
    V::Remainder: Views<'a>,
{
    type Canonical = (
        &'a C,
        <R as ContainsViewsInner<'a, <V as Get<&'a C, I>>::Remainder, P, IS>>::Canonical,
    );
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (&'a mut Contained, P), (I, IS)> for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as Get<&'a mut C, I>>::Remainder, P, IS>,
    V: Views<'a> + Get<&'a mut C, I>,
    V::Remainder: Views<'a>,
{
    type Canonical = (
        &'a mut C,
        <R as ContainsViewsInner<'a, <V as Get<&'a mut C, I>>::Remainder, P, IS>>::Canonical,
    );
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (Option<&'a Contained>, P), (I, IS)>
    for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as Get<Option<&'a C>, I>>::Remainder, P, IS>,
    V: Views<'a> + Get<Option<&'a C>, I>,
    V::Remainder: Views<'a>,
{
    type Canonical = (
        Option<&'a C>,
        <R as ContainsViewsInner<'a, <V as Get<Option<&'a C>, I>>::Remainder, P, IS>>::Canonical,
    );
}

impl<'a, C, I, IS, P, R, V> ContainsViewsInner<'a, V, (Option<&'a mut Contained>, P), (I, IS)>
    for (C, R)
where
    C: Component,
    R: ContainsViewsInner<'a, <V as Get<Option<&'a mut C>, I>>::Remainder, P, IS>,
    V: Views<'a> + Get<Option<&'a mut C>, I>,
    V::Remainder: Views<'a>,
{
    type Canonical = (
        Option<&'a mut C>,
        <R as ContainsViewsInner<
            'a,
            <V as Get<Option<&'a mut C>, I>>::Remainder,
            P,
            IS,
        >>::Canonical,
    );
}

impl<'a, I, IS, P, V, R> ContainsViewsInner<'a, V, (Contained, P), (I, IS)>
    for (EntityIdentifierMarker, R)
where
    R: ContainsViewsInner<'a, <V as Get<entity::Identifier, I>>::Remainder, P, IS>,
    V: Views<'a> + Get<entity::Identifier, I>,
    V::Remainder: Views<'a>,
{
    type Canonical = (
        entity::Identifier,
        <R as ContainsViewsInner<
            'a,
            <V as Get<entity::Identifier, I>>::Remainder,
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
