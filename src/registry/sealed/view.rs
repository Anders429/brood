#[cfg(feature = "rayon")]
use crate::query::view::{
    claim,
    Claim,
};
use crate::{
    archetype,
    component::Component,
    query::{
        view,
        view::{
            Views,
            ViewsSealed,
        },
    },
    registry,
    registry::{
        contains::{
            Contained,
            NotContained,
            Null,
        },
        Registry,
    },
};
use core::{
    iter,
    mem::MaybeUninit,
    slice,
};
use either::Either;

pub trait CanonicalViews<'a, V, P>: Registry
where
    V: Views<'a>,
{
    /// # Safety
    ///
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`, ordered for the archetype identified by `archetype_identifier`.
    unsafe fn view<R>(
        columns: &[(*mut u8, usize)],
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R>,
    ) -> V::Results
    where
        R: Registry;

    /// # Safety
    ///
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`, ordered for the archetype identified by `archetype_identifier`.
    ///
    /// Additionally, `index` must be a less than `length`.
    unsafe fn view_one<R>(
        index: usize,
        columns: &[(*mut u8, usize)],
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R>,
    ) -> V
    where
        R: Registry;

    unsafe fn view_one_maybe_uninit<R>(
        index: usize,
        columns: &[(*mut u8, usize)],
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R>,
    ) -> V::MaybeUninit
    where
        R: Registry;

    /// Return the dynamic claims over the components borrowed by the `Views`.
    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims;

    /// Return the indices for each view into the registry.
    fn indices<R>() -> V::Indices
    where
        R: registry::Length;
}

impl<'a> CanonicalViews<'a, view::Null, Null> for registry::Null {
    unsafe fn view<R>(
        _columns: &[(*mut u8, usize)],
        length: usize,
        _archetype_identifier: archetype::identifier::Iter<R>,
    ) -> <view::Null as ViewsSealed<'a>>::Results
    where
        R: Registry,
    {
        iter::repeat(view::Null).take(length)
    }

    unsafe fn view_one<R>(
        _index: usize,
        _columns: &[(*mut u8, usize)],
        _length: usize,
        _archetype_identifier: archetype::identifier::Iter<R>,
    ) -> view::Null
    where
        R: Registry,
    {
        view::Null
    }

    unsafe fn view_one_maybe_uninit<R>(
        _index: usize,
        _columns: &[(*mut u8, usize)],
        _length: usize,
        _archetype_identifier: archetype::identifier::Iter<R>,
    ) -> view::Null
    where
        R: Registry,
    {
        view::Null
    }

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims {
        claim::Null
    }

    fn indices<R>() -> view::Null
    where
        R: registry::Length,
    {
        view::Null
    }
}

impl<'a, C, P, R, V> CanonicalViews<'a, (&'a C, V), (&'a Contained, P)> for (C, R)
where
    C: Component,
    R: CanonicalViews<'a, V, P>,
    V: Views<'a>,
{
    unsafe fn view<R_>(
        columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(&'a C, V) as ViewsSealed<'a>>::Results
    where
        R_: Registry,
    {
        archetype_identifier.next();
        (
            // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
            // `length` for the currently viewed component `C`.
            unsafe {
                slice::from_raw_parts::<'a, C>(columns.get_unchecked(0).0.cast::<C>(), length)
            }
            .iter(),
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`.
            unsafe { R::view(columns.get_unchecked(1..), length, archetype_identifier) },
        )
    }

    unsafe fn view_one<R_>(
        index: usize,
        columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> (&'a C, V)
    where
        R_: Registry,
    {
        archetype_identifier.next();
        (
            // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
            // `length` for the currently viewed component `C`. Consequentially, `index` is a valid
            // index into this `Vec<C>`.
            unsafe {
                slice::from_raw_parts::<'a, C>(columns.get_unchecked(0).0.cast::<C>(), length)
                    .get_unchecked(index)
            },
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`. `index` is guaranteed to be less than
            // `length`.
            unsafe {
                R::view_one(
                    index,
                    columns.get_unchecked(1..),
                    length,
                    archetype_identifier,
                )
            },
        )
    }

    unsafe fn view_one_maybe_uninit<R_>(
        index: usize,
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> (MaybeUninit<&'a C>, V::MaybeUninit)
    where
        R_: Registry,
    {
        (
            // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
            if unsafe { archetype_identifier.next().unwrap_unchecked() } {
                // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                // size `length` for the currently viewed component `C`. Consequentially, `index`
                // is a valid index into this `Vec<C>`.
                MaybeUninit::new(unsafe {
                    slice::from_raw_parts(
                        {
                            let column = columns.get_unchecked(0);
                            columns = columns.get_unchecked(1..);
                            column
                        }
                        .0
                        .cast::<C>(),
                        length,
                    )
                    .get_unchecked(index)
                })
            } else {
                MaybeUninit::uninit()
            },
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`. `index` is guaranteed to be less than
            // `length`.
            unsafe { R::view_one_maybe_uninit(index, columns, length, archetype_identifier) },
        )
    }

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims {
        (Claim::Immutable, R::claims())
    }

    fn indices<R_>() -> (usize, V::Indices)
    where
        R_: registry::Length,
    {
        (R_::LEN - R::LEN - 1, R::indices::<R_>())
    }
}

impl<'a, C, P, R, V> CanonicalViews<'a, (&'a mut C, V), (&'a mut Contained, P)> for (C, R)
where
    C: Component,
    R: CanonicalViews<'a, V, P>,
    V: Views<'a>,
{
    unsafe fn view<R_>(
        columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(&'a mut C, V) as ViewsSealed<'a>>::Results
    where
        R_: Registry,
    {
        archetype_identifier.next();
        (
            // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
            // `length` for the currently viewed component `C`.
            unsafe {
                slice::from_raw_parts_mut::<'a, C>(columns.get_unchecked(0).0.cast::<C>(), length)
            }
            .iter_mut(),
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`.
            unsafe { R::view(columns.get_unchecked(1..), length, archetype_identifier) },
        )
    }

    unsafe fn view_one<R_>(
        index: usize,
        columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> (&'a mut C, V)
    where
        R_: Registry,
    {
        archetype_identifier.next();
        (
            // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
            // `length` for the currently viewed component `C`. Consequentially, `index` is a valid
            // index into this `Vec<C>`.
            unsafe {
                slice::from_raw_parts_mut::<'a, C>(columns.get_unchecked(0).0.cast::<C>(), length)
                    .get_unchecked_mut(index)
            },
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`. `index` is guaranteed to be less than
            // `length`.
            unsafe {
                R::view_one(
                    index,
                    columns.get_unchecked(1..),
                    length,
                    archetype_identifier,
                )
            },
        )
    }

    unsafe fn view_one_maybe_uninit<R_>(
        index: usize,
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> (MaybeUninit<&'a mut C>, V::MaybeUninit)
    where
        R_: Registry,
    {
        (
            // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
            if unsafe { archetype_identifier.next().unwrap_unchecked() } {
                // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                // size `length` for the currently viewed component `C`. Consequentially, `index`
                // is a valid index into this `Vec<C>`.
                MaybeUninit::new(unsafe {
                    slice::from_raw_parts_mut(
                        {
                            let column = columns.get_unchecked(0);
                            columns = columns.get_unchecked(1..);
                            column
                        }
                        .0
                        .cast::<C>(),
                        length,
                    )
                    .get_unchecked_mut(index)
                })
            } else {
                MaybeUninit::uninit()
            },
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`. `index` is guaranteed to be less than
            // `length`.
            unsafe { R::view_one_maybe_uninit(index, columns, length, archetype_identifier) },
        )
    }

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims {
        (Claim::Mutable, R::claims())
    }

    fn indices<R_>() -> (usize, V::Indices)
    where
        R_: registry::Length,
    {
        (R_::LEN - R::LEN - 1, R::indices::<R_>())
    }
}

#[allow(clippy::unnecessary_wraps)]
fn wrap_some<T>(val: T) -> Option<T> {
    Some(val)
}

impl<'a, C, P, R, V> CanonicalViews<'a, (Option<&'a C>, V), (Option<&'a Contained>, P)> for (C, R)
where
    C: Component,
    R: CanonicalViews<'a, V, P>,
    V: Views<'a>,
{
    unsafe fn view<R_>(
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(Option<&'a C>, V) as ViewsSealed<'a>>::Results
    where
        R_: Registry,
    {
        (
            // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
            if unsafe { archetype_identifier.next().unwrap_unchecked() } {
                Either::Right(
                    // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                    // size `length` for the currently viewed component `C`.
                    unsafe {
                        slice::from_raw_parts(
                            {
                                let column = columns.get_unchecked(0);
                                columns = columns.get_unchecked(1..);
                                column
                            }
                            .0
                            .cast::<C>(),
                            length,
                        )
                    }
                    .iter()
                    .map(wrap_some),
                )
            } else {
                Either::Left(iter::repeat(None).take(length))
            },
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`.
            unsafe { R::view(columns, length, archetype_identifier) },
        )
    }

    unsafe fn view_one<R_>(
        index: usize,
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> (Option<&'a C>, V)
    where
        R_: Registry,
    {
        (
            // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
            unsafe { archetype_identifier.next().unwrap_unchecked() }.then(||
                // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                // size `length` for the currently viewed component `C`. Consequentially, `index`
                // is a valid index into this `Vec<C>`.
                unsafe {
                slice::from_raw_parts(
                    {
                        let column = columns.get_unchecked(0);
                        columns = columns.get_unchecked(1..);
                        column
                    }
                    .0
                    .cast::<C>(),
                    length,
                )
                .get_unchecked(index)
            }),
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`. `index` is guaranteed to be less than
            // `length`.
            unsafe { R::view_one(index, columns, length, archetype_identifier) },
        )
    }

    unsafe fn view_one_maybe_uninit<R_>(
        index: usize,
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> (Option<&'a C>, V::MaybeUninit)
    where
        R_: Registry,
    {
        (
            // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
            unsafe { archetype_identifier.next().unwrap_unchecked() }.then(||
                // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                // size `length` for the currently viewed component `C`. Consequentially, `index`
                // is a valid index into this `Vec<C>`.
                unsafe {
                slice::from_raw_parts(
                    {
                        let column = columns.get_unchecked(0);
                        columns = columns.get_unchecked(1..);
                        column
                    }
                    .0
                    .cast::<C>(),
                    length,
                )
                .get_unchecked(index)
            }),
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`. `index` is guaranteed to be less than
            // `length`.
            unsafe { R::view_one_maybe_uninit(index, columns, length, archetype_identifier) },
        )
    }

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims {
        (Claim::Immutable, R::claims())
    }

    fn indices<R_>() -> (usize, V::Indices)
    where
        R_: registry::Length,
    {
        (R_::LEN - R::LEN - 1, R::indices::<R_>())
    }
}

impl<'a, C, P, R, V> CanonicalViews<'a, (Option<&'a mut C>, V), (Option<&'a mut Contained>, P)>
    for (C, R)
where
    C: Component,
    R: CanonicalViews<'a, V, P>,
    V: Views<'a>,
{
    unsafe fn view<R_>(
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(Option<&'a mut C>, V) as ViewsSealed<'a>>::Results
    where
        R_: Registry,
    {
        fn none<'a, C>() -> Option<&'a mut C> {
            None
        }

        (
            // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
            if unsafe { archetype_identifier.next().unwrap_unchecked() } {
                Either::Right(
                    // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                    // size `length` for the currently viewed component `C`.
                    unsafe {
                        slice::from_raw_parts_mut(
                            {
                                let column = columns.get_unchecked(0);
                                columns = columns.get_unchecked(1..);
                                column
                            }
                            .0
                            .cast::<C>(),
                            length,
                        )
                    }
                    .iter_mut()
                    .map(wrap_some),
                )
            } else {
                Either::Left(iter::repeat_with(none as fn() -> Option<&'a mut C>).take(length))
            },
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`.
            unsafe { R::view(columns, length, archetype_identifier) },
        )
    }

    unsafe fn view_one<R_>(
        index: usize,
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> (Option<&'a mut C>, V)
    where
        R_: Registry,
    {
        (
            // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
            unsafe { archetype_identifier.next().unwrap_unchecked() }.then(||
                // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                // size `length` for the currently viewed component `C`. Consequentially, `index`
                // is a valid index into this `Vec<C>`.
                unsafe {
                    slice::from_raw_parts_mut(
                        {
                            let column = columns.get_unchecked(0);
                            columns = columns.get_unchecked(1..);
                            column
                        }
                        .0
                        .cast::<C>(),
                        length,
                    )
                    .get_unchecked_mut(index)
                }),
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`. `index` is guaranteed to be less than
            // `length`.
            unsafe { R::view_one(index, columns, length, archetype_identifier) },
        )
    }

    unsafe fn view_one_maybe_uninit<R_>(
        index: usize,
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> (Option<&'a mut C>, V::MaybeUninit)
    where
        R_: Registry,
    {
        (
            // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
            unsafe { archetype_identifier.next().unwrap_unchecked() }.then(||
                // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                // size `length` for the currently viewed component `C`. Consequentially, `index`
                // is a valid index into this `Vec<C>`.
                unsafe {
                    slice::from_raw_parts_mut(
                        {
                            let column = columns.get_unchecked(0);
                            columns = columns.get_unchecked(1..);
                            column
                        }
                        .0
                        .cast::<C>(),
                        length,
                    )
                    .get_unchecked_mut(index)
                }),
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`. `index` is guaranteed to be less than
            // `length`.
            unsafe { R::view_one_maybe_uninit(index, columns, length, archetype_identifier) },
        )
    }

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims {
        (Claim::Mutable, R::claims())
    }

    fn indices<R_>() -> (usize, V::Indices)
    where
        R_: registry::Length,
    {
        (R_::LEN - R::LEN - 1, R::indices::<R_>())
    }
}

impl<'a, C, P, R, V> CanonicalViews<'a, V, (NotContained, P)> for (C, R)
where
    C: Component,
    R: CanonicalViews<'a, V, P>,
    V: Views<'a>,
{
    unsafe fn view<R_>(
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> V::Results
    where
        R_: Registry,
    {
        // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
        if unsafe { archetype_identifier.next().unwrap_unchecked() } {
            // SAFETY: Since `archetype_identifier` has this component set, there is guaranteed to
            // be at least one entry in `columns`.
            unsafe {
                columns = columns.get_unchecked(1..);
            }
        }
        // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
        // for valid `Vec<C>`s of length `length` for each of the remaining components
        // identified by `archetype_identifier`.
        unsafe { R::view(columns, length, archetype_identifier) }
    }

    unsafe fn view_one<R_>(
        index: usize,
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> V
    where
        R_: Registry,
    {
        // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
        if unsafe { archetype_identifier.next().unwrap_unchecked() } {
            // SAFETY: Since `archetype_identifier` has this component set, there is guaranteed to
            // be at least one entry in `columns`.
            unsafe {
                columns = columns.get_unchecked(1..);
            }
        }
        // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
        // for valid `Vec<C>`s of length `length` for each of the remaining components
        // identified by `archetype_identifier`. `index` is guaranteed to be less than `length`.
        unsafe { R::view_one(index, columns, length, archetype_identifier) }
    }

    unsafe fn view_one_maybe_uninit<R_>(
        index: usize,
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> V::MaybeUninit
    where
        R_: Registry,
    {
        // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
        if unsafe { archetype_identifier.next().unwrap_unchecked() } {
            // SAFETY: Since `archetype_identifier` has this component set, there is guaranteed to
            // be at least one entry in `columns`.
            unsafe {
                columns = columns.get_unchecked(1..);
            }
        }
        // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
        // for valid `Vec<C>`s of length `length` for each of the remaining components
        // identified by `archetype_identifier`. `index` is guaranteed to be less than `length`.
        unsafe { R::view_one_maybe_uninit(index, columns, length, archetype_identifier) }
    }

    #[cfg(feature = "rayon")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "rayon")))]
    fn claims() -> Self::Claims {
        (Claim::None, R::claims())
    }

    fn indices<R_>() -> V::Indices
    where
        R_: registry::Length,
    {
        R::indices::<R_>()
    }
}
