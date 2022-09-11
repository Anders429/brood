use crate::{
    archetype,
    component::Component,
    entity,
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
    unsafe fn view<R>(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        archetype_identifier: archetype::identifier::Iter<R>,
    ) -> V::Results
    where
        R: Registry;
}

impl<'a> CanonicalViews<'a, view::Null, Null> for registry::Null {
    unsafe fn view<R>(
        _columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
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
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(&'a C, V) as ViewsSeal<'a>>::Results
    where
        R_: Registry,
    {
        archetype_identifier.next();
        (
            unsafe {
                slice::from_raw_parts::<'a, C>(columns.get_unchecked(0).0.cast::<C>(), length)
            }
            .iter(),
            unsafe {
                R::view(
                    columns.get_unchecked(1..),
                    entity_identifiers,
                    length,
                    archetype_identifier,
                )
            },
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
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(&'a mut C, V) as ViewsSeal<'a>>::Results
    where
        R_: Registry,
    {
        archetype_identifier.next();
        (
            unsafe {
                slice::from_raw_parts_mut::<'a, C>(columns.get_unchecked(0).0.cast::<C>(), length)
            }
            .iter_mut(),
            unsafe {
                R::view(
                    columns.get_unchecked(1..),
                    entity_identifiers,
                    length,
                    archetype_identifier,
                )
            },
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
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> <(Option<&'a C>, V) as ViewsSeal<'a>>::Results
    where
        R_: Registry,
    {
        (
            if unsafe { archetype_identifier.next().unwrap_unchecked() } {
                Either::Right(
                    unsafe {
                        slice::from_raw_parts(columns.get_unchecked(0).0.cast::<C>(), length)
                    }
                    .iter()
                    .map(wrap_some),
                )
            } else {
                Either::Left(iter::repeat(None).take(length))
            },
            unsafe {
                R::view(
                    columns.get_unchecked(1..),
                    entity_identifiers,
                    length,
                    archetype_identifier,
                )
            },
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
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
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
            if unsafe { archetype_identifier.next().unwrap_unchecked() } {
                Either::Right(
                    unsafe {
                        slice::from_raw_parts_mut(columns.get_unchecked(0).0.cast::<C>(), length)
                    }
                    .iter_mut()
                    .map(wrap_some),
                )
            } else {
                Either::Left(iter::repeat_with(none as fn() -> Option<&'a mut C>).take(length))
            },
            unsafe {
                R::view(
                    columns.get_unchecked(1..),
                    entity_identifiers,
                    length,
                    archetype_identifier,
                )
            },
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
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        mut archetype_identifier: archetype::identifier::Iter<R_>,
    ) -> V::Results
    where
        R_: Registry,
    {
        if unsafe {archetype_identifier.next().unwrap_unchecked()} {
            unsafe {columns = columns.get_unchecked(1..);}
        }
        unsafe { R::view(columns, entity_identifiers, length, archetype_identifier) }
    }
}
