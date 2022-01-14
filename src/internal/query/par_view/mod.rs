mod repeat;

use crate::{
    component::Component,
    entity,
    query::{
        result,
        view::{Null, View, Views},
    },
};
use core::any::TypeId;
use hashbrown::HashMap;
use rayon::{
    iter,
    iter::{
        Either, IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator,
        ParallelIterator,
    },
    slice,
};
use repeat::RepeatNone;

pub trait ParViewSeal<'a>: View<'a> {
    type ParResult: IndexedParallelIterator;

    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::ParResult;
}

impl<'a, C> ParViewSeal<'a> for &C
where
    C: Component + Sync,
{
    type ParResult = slice::Iter<'a, C>;

    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::ParResult {
        core::slice::from_raw_parts::<'a, C>(
            columns
                .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                .0 as *mut C,
            length,
        )
        .par_iter()
    }
}

impl<'a, C> ParViewSeal<'a> for &mut C
where
    C: Component + Send,
{
    type ParResult = slice::IterMut<'a, C>;

    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::ParResult {
        core::slice::from_raw_parts_mut::<'a, C>(
            columns
                .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                .0 as *mut C,
            length,
        )
        .par_iter_mut()
    }
}

fn wrap_some<T>(val: T) -> Option<T> {
    Some(val)
}

impl<'a, C> ParViewSeal<'a> for Option<&C>
where
    C: Component + Sync,
{
    type ParResult = Either<
        iter::RepeatN<Option<&'a C>>,
        iter::Map<slice::Iter<'a, C>, fn(&'a C) -> Option<&'a C>>,
    >;

    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::ParResult {
        match component_map.get(&TypeId::of::<C>()) {
            Some(index) => Either::Right(
                core::slice::from_raw_parts(columns.get_unchecked(*index).0 as *mut C, length)
                    .par_iter()
                    .map(wrap_some),
            ),
            None => Either::Left(iter::repeat(None).take(length)),
        }
    }
}

impl<'a, C> ParViewSeal<'a> for Option<&mut C>
where
    C: Component + Send,
{
    type ParResult = Either<
        RepeatNone<&'a mut C>,
        iter::Map<slice::IterMut<'a, C>, fn(&'a mut C) -> Option<&'a mut C>>,
    >;

    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::ParResult {
        match component_map.get(&TypeId::of::<C>()) {
            Some(index) => Either::Right(
                core::slice::from_raw_parts_mut(columns.get_unchecked(*index).0 as *mut C, length)
                    .par_iter_mut()
                    .map(wrap_some),
            ),
            None => Either::Left(RepeatNone::new(length)),
        }
    }
}

impl<'a> ParViewSeal<'a> for entity::Identifier {
    type ParResult = iter::Cloned<slice::Iter<'a, Self>>;

    unsafe fn par_view(
        _columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        _component_map: &HashMap<TypeId, usize>,
    ) -> Self::ParResult {
        core::slice::from_raw_parts_mut::<'a, Self>(entity_identifiers.0, length)
            .par_iter()
            .cloned()
    }
}

pub trait ParViewsSeal<'a>: Views<'a> {
    type ParResults: IndexedParallelIterator;

    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::ParResults;
}

impl<'a> ParViewsSeal<'a> for Null {
    type ParResults = iter::RepeatN<result::Null>;

    unsafe fn par_view(
        _columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        _length: usize,
        _component_map: &HashMap<TypeId, usize>,
    ) -> Self::ParResults {
        iter::repeatn(result::Null, usize::MAX)
    }
}

impl<'a, V, W> ParViewsSeal<'a> for (V, W)
where
    V: ParViewSeal<'a>,
    W: ParViewsSeal<'a>,
{
    type ParResults = iter::Zip<V::ParResult, W::ParResults>;

    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::ParResults {
        V::par_view(columns, entity_identifiers, length, component_map).zip(W::par_view(
            columns,
            entity_identifiers,
            length,
            component_map,
        ))
    }
}
