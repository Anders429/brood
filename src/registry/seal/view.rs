use crate::{
    archetype,
    component::Component,
    query::{
        view,
        view::{Views, ViewsSeal},
    },
    registry,
    registry::{
        contains::{Contained, NotContained, Null},
        Registry,
    },
};
use core::{iter, slice};
use either::Either;

pub trait CanonicalViews<'a, V, P>
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
}

impl<'a> CanonicalViews<'a, view::Null, Null> for registry::Null {
    unsafe fn view<R>(
        _columns: &[(*mut u8, usize)],
        _length: usize,
        _archetype_identifier: archetype::identifier::Iter<R>,
    ) -> <view::Null as ViewsSeal<'a>>::Results
    where
        R: Registry,
    {
        iter::repeat(view::Null)
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
    ) -> <(&'a C, V) as ViewsSeal<'a>>::Results
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
    ) -> <(&'a mut C, V) as ViewsSeal<'a>>::Results
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
    ) -> <(Option<&'a C>, V) as ViewsSeal<'a>>::Results
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
                        slice::from_raw_parts({let column = columns.get_unchecked(0); columns = columns.get_unchecked(1..); column}.0.cast::<C>(), length)
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
    ) -> <(Option<&'a mut C>, V) as ViewsSeal<'a>>::Results
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
                        slice::from_raw_parts_mut({let column = columns.get_unchecked(0); columns = columns.get_unchecked(1..); column}.0.cast::<C>(), length)
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
}
