use crate::{
    archetype,
    component::Component,
    query::{
        view,
        view::{
            ParViews,
            ParViewsSeal,
            RepeatNone,
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
use rayon::{
    iter,
    iter::{
        Either,
        IntoParallelRefIterator,
        IntoParallelRefMutIterator,
        ParallelIterator,
    },
};

pub trait CanonicalParViews<'a, V, P>
where
    V: ParViews<'a>,
{
    /// # Safety
    ///
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`, ordered for the archetype identified by `archetype_identifier`.
    unsafe fn par_view<R>(
        columns: &[(*mut u8, usize)],
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R>,
    ) -> V::ParResults
    where
        R: Registry;
}

impl<'a> CanonicalParViews<'a, view::Null, Null> for registry::Null {
    unsafe fn par_view<R>(
        _columns: &[(*mut u8, usize)],
        _length: usize,
        _archetype_identifier: archetype::identifier::Iter<R>,
    ) -> <view::Null as ParViewsSeal<'a>>::ParResults
    where
        R: Registry,
    {
        iter::repeatn(view::Null, usize::MAX)
    }
}

impl<'a, C, P, R, V> CanonicalParViews<'a, (&'a C, V), (&'a Contained, P)> for (C, R)
where
    C: Component + Sync,
    R: CanonicalParViews<'a, V, P>,
    V: ParViews<'a>,
{
    unsafe fn par_view<R_>(
        columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(&'a C, V) as ParViewsSeal<'a>>::ParResults
    where
        R_: Registry,
    {
        archetype_identifier.next();
        (
            // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
            // `length` for the currently viewed component `C`.
            unsafe {
                core::slice::from_raw_parts::<'a, C>(columns.get_unchecked(0).0.cast::<C>(), length)
            }
            .par_iter(),
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`.
            unsafe { R::par_view(columns.get_unchecked(1..), length, archetype_identifier) },
        )
    }
}

impl<'a, C, P, R, V> CanonicalParViews<'a, (&'a mut C, V), (&'a mut Contained, P)> for (C, R)
where
    C: Component + Send,
    R: CanonicalParViews<'a, V, P>,
    V: ParViews<'a>,
{
    unsafe fn par_view<R_>(
        columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(&'a mut C, V) as ParViewsSeal<'a>>::ParResults
    where
        R_: Registry,
    {
        archetype_identifier.next();
        (
            // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
            // `length` for the currently viewed component `C`.
            unsafe {
                core::slice::from_raw_parts_mut::<'a, C>(
                    columns.get_unchecked(0).0.cast::<C>(),
                    length,
                )
            }
            .par_iter_mut(),
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`.
            unsafe { R::par_view(columns.get_unchecked(1..), length, archetype_identifier) },
        )
    }
}

#[allow(clippy::unnecessary_wraps)]
fn wrap_some<T>(val: T) -> Option<T> {
    Some(val)
}

impl<'a, C, P, R, V> CanonicalParViews<'a, (Option<&'a C>, V), (Option<&'a Contained>, P)>
    for (C, R)
where
    C: Component + Sync,
    R: CanonicalParViews<'a, V, P>,
    V: ParViews<'a>,
{
    unsafe fn par_view<R_>(
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(Option<&'a C>, V) as ParViewsSeal<'a>>::ParResults
    where
        R_: Registry,
    {
        (
            // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
            if unsafe { archetype_identifier.next().unwrap_unchecked() } {
                Either::Right(
                    // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                    // size `length`. Since `component_map` contains an entry for the given
                    // component `C`'s entry in `columns`, then the column
                    // obtained here can be interpreted as a slice of type `C`
                    // of size `length`.
                    unsafe {
                        core::slice::from_raw_parts(
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
                    .par_iter()
                    .map(wrap_some),
                )
            } else {
                Either::Left(iter::repeat(None).take(length))
            },
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`.
            unsafe { R::par_view(columns, length, archetype_identifier) },
        )
    }
}

impl<'a, C, P, R, V> CanonicalParViews<'a, (Option<&'a mut C>, V), (Option<&'a mut Contained>, P)>
    for (C, R)
where
    C: Component + Send,
    R: CanonicalParViews<'a, V, P>,
    V: ParViews<'a>,
{
    unsafe fn par_view<R_>(
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(Option<&'a mut C>, V) as ParViewsSeal<'a>>::ParResults
    where
        R_: Registry,
    {
        (
            // SAFETY: `archetype_identifier` is guaranteed to have at least one element remaining.
            if unsafe { archetype_identifier.next().unwrap_unchecked() } {
                Either::Right(
                    // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                    // size `length`. Since `component_map` contains an entry for the given
                    // component `C`'s entry in `columns`, then the column
                    // obtained here can be interpreted as a slice of type `C`
                    // of size `length`.
                    unsafe {
                        core::slice::from_raw_parts_mut(
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
                    .par_iter_mut()
                    .map(wrap_some),
                )
            } else {
                Either::Left(RepeatNone::new(length))
            },
            // SAFETY: The remaining components in `columns` are guaranteed to contain raw parts
            // for valid `Vec<C>`s of length `length` for each of the remaining components
            // identified by `archetype_identifier`.
            unsafe { R::par_view(columns, length, archetype_identifier) },
        )
    }
}

impl<'a, C, P, R, V> CanonicalParViews<'a, V, (NotContained, P)> for (C, R)
where
    C: Component,
    R: CanonicalParViews<'a, V, P>,
    V: ParViews<'a>,
{
    unsafe fn par_view<R_>(
        mut columns: &[(*mut u8, usize)],
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> V::ParResults
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
        unsafe { R::par_view(columns, length, archetype_identifier) }
    }
}
