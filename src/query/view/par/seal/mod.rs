mod repeat;

use crate::{
    component::Component,
    entity,
    query::{
        view::{Null, View, Views},
    },
};
use core::any::TypeId;
use fnv::FnvBuildHasher;
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
    type ParResult: IndexedParallelIterator<Item = Self>;

    /// # Safety
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`. Each of those components `C` must have an entry in `component_map`,
    /// paired with the correct index corresponding to that component's entry in `columns`.
    ///
    /// `entity_identifiers` must contain the raw parts for a valid `Vec<entity::Identifier` of
    /// size `length`.
    ///
    /// `component_map` must contain an entry for every component `C` that is viewed by this
    /// `ParView`, and that entry must contain the index for the column of type `C` in `columns`.
    /// Note that it is not required for optionally viewed components to be contained in the
    /// `component_map`.
    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::ParResult;
}

impl<'a, C> ParViewSeal<'a> for &'a C
where
    C: Component + Sync,
{
    type ParResult = slice::Iter<'a, C>;

    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::ParResult {
        // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
        // `length`. Since `component_map` contains an entry for the given component `C`'s entry in
        // `columns`, then the column obtained here can be interpreted as a slice of type `C` of
        // size `length`.
        unsafe {
            core::slice::from_raw_parts::<'a, C>(
                columns
                    .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                    .0
                    .cast::<C>(),
                length,
            )
        }
        .par_iter()
    }
}

impl<'a, C> ParViewSeal<'a> for &'a mut C
where
    C: Component + Send,
{
    type ParResult = slice::IterMut<'a, C>;

    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::ParResult {
        // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of size
        // `length`. Since `component_map` contains an entry for the given component `C`'s entry in
        // `columns`, then the column obtained here can be interpreted as a slice of type `C` of
        // size `length`.
        unsafe {
            core::slice::from_raw_parts_mut::<'a, C>(
                columns
                    .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                    .0
                    .cast::<C>(),
                length,
            )
        }
        .par_iter_mut()
    }
}

#[allow(clippy::unnecessary_wraps)]
fn wrap_some<T>(val: T) -> Option<T> {
    Some(val)
}

impl<'a, C> ParViewSeal<'a> for Option<&'a C>
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
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::ParResult {
        match component_map.get(&TypeId::of::<C>()) {
            Some(index) => Either::Right(
                // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                // size `length`. Since `component_map` contains an entry for the given component
                // `C`'s entry in `columns`, then the column obtained here can be interpreted as a
                // slice of type `C` of size `length`.
                unsafe {
                    core::slice::from_raw_parts(columns.get_unchecked(*index).0.cast::<C>(), length)
                }
                .par_iter()
                .map(wrap_some),
            ),
            None => Either::Left(iter::repeat(None).take(length)),
        }
    }
}

impl<'a, C> ParViewSeal<'a> for Option<&'a mut C>
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
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::ParResult {
        match component_map.get(&TypeId::of::<C>()) {
            Some(index) => Either::Right(
                // SAFETY: `columns` is guaranteed to contain raw parts for a valid `Vec<C>` of
                // size `length`. Since `component_map` contains an entry for the given component
                // `C`'s entry in `columns`, then the column obtained here can be interpreted as a
                // slice of type `C` of size `length`.
                unsafe {
                    core::slice::from_raw_parts_mut(
                        columns.get_unchecked(*index).0.cast::<C>(),
                        length,
                    )
                }
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
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::ParResult {
        // SAFETY: `entity_identifiers` is guaranteed to contain the raw parts for a valid
        // `Vec<entity::Identifier>` of size `length`.
        unsafe { core::slice::from_raw_parts_mut::<'a, Self>(entity_identifiers.0, length) }
            .par_iter()
            .cloned()
    }
}

pub trait ParViewsSeal<'a>: Views<'a> {
    type ParResults: IndexedParallelIterator<Item = Self>;

    /// # Safety
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`. Each of those components `C` must have an entry in `component_map`,
    /// paired with the correct index corresponding to that component's entry in `columns`.
    ///
    /// `entity_identifiers` must contain the raw parts for a valid `Vec<entity::Identifier` of
    /// size `length`.
    ///
    /// `component_map` must contain an entry for every component `C` that is viewed by this
    /// `ParViews`, and that entry must contain the index for the column of type `C` in `columns`.
    /// Note that it is not required for optionally viewed components to be contained in the
    /// `component_map`.
    unsafe fn par_view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::ParResults;
}

impl<'a> ParViewsSeal<'a> for Null {
    type ParResults = iter::RepeatN<Null>;

    unsafe fn par_view(
        _columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        _length: usize,
        _component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::ParResults {
        iter::repeatn(Null, usize::MAX)
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
        component_map: &HashMap<TypeId, usize, FnvBuildHasher>,
    ) -> Self::ParResults {
        // SAFETY: The safety guarantees of this method are the exact what are required by the
        // safety guarantees of both `V::par_view()` and `W::par_view()`.
        unsafe {
            V::par_view(columns, entity_identifiers, length, component_map).zip(W::par_view(
                columns,
                entity_identifiers,
                length,
                component_map,
            ))
        }
    }
}
