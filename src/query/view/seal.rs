use crate::{
    component::Component,
    entity,
    query::{
        claim::Claim,
        result,
        view::{AssertionBuffer, Null},
    },
};
use core::{any::TypeId, iter, slice};
use either::Either;
use hashbrown::HashMap;

pub trait ViewSeal<'a>: Claim {
    type Result: Iterator;

    /// # Safety
    /// Each tuple in `columns` must contain the raw parts for a valid `Vec<C>` of size `length`
    /// for components `C`. Each of those components `C` must have an entry in `component_map`,
    /// paired with the correct index corresponding to that component's entry in `columns`.
    ///
    /// `entity_identifiers` must contain the raw parts for a valid `Vec<entity::Identifier` of
    /// size `length`.
    ///
    /// `component_map` must contain an entry for every component `C` that is viewed by this
    /// `View`.
    unsafe fn view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result;

    fn assert_claim(buffer: &mut AssertionBuffer);
}

impl<'a, C> ViewSeal<'a> for &C
where
    C: Component,
{
    type Result = slice::Iter<'a, C>;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result {
        unsafe {
            slice::from_raw_parts::<'a, C>(
                columns
                    .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                    .0
                    .cast::<C>(),
                length,
            )
        }
        .iter()
    }

    fn assert_claim(buffer: &mut AssertionBuffer) {
        buffer.claim_immutable::<C>();
    }
}

impl<'a, C> ViewSeal<'a> for &mut C
where
    C: Component,
{
    type Result = slice::IterMut<'a, C>;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result {
        unsafe {
            slice::from_raw_parts_mut::<'a, C>(
                columns
                    .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                    .0
                    .cast::<C>(),
                length,
            )
        }
        .iter_mut()
    }

    fn assert_claim(buffer: &mut AssertionBuffer) {
        buffer.claim_mutable::<C>();
    }
}

#[allow(clippy::unnecessary_wraps)]
fn wrap_some<T>(val: T) -> Option<T> {
    Some(val)
}

impl<'a, C> ViewSeal<'a> for Option<&C>
where
    C: Component,
{
    type Result = Either<
        iter::Take<iter::Repeat<Option<&'a C>>>,
        iter::Map<slice::Iter<'a, C>, fn(&'a C) -> Option<&'a C>>,
    >;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result {
        match component_map.get(&TypeId::of::<C>()) {
            Some(index) => Either::Right(
                unsafe {
                    slice::from_raw_parts(columns.get_unchecked(*index).0.cast::<C>(), length)
                }
                .iter()
                .map(wrap_some),
            ),
            None => Either::Left(iter::repeat(None).take(length)),
        }
    }

    fn assert_claim(buffer: &mut AssertionBuffer) {
        buffer.claim_immutable::<C>();
    }
}

impl<'a, C> ViewSeal<'a> for Option<&mut C>
where
    C: Component,
{
    type Result = Either<
        iter::Take<iter::RepeatWith<fn() -> Option<&'a mut C>>>,
        iter::Map<slice::IterMut<'a, C>, fn(&'a mut C) -> Option<&'a mut C>>,
    >;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result {
        fn none<'a, C>() -> Option<&'a mut C> {
            None
        }

        match component_map.get(&TypeId::of::<C>()) {
            Some(index) => Either::Right(
                unsafe {
                    slice::from_raw_parts_mut(columns.get_unchecked(*index).0.cast::<C>(), length)
                }
                .iter_mut()
                .map(wrap_some),
            ),
            None => Either::Left(iter::repeat_with(none as fn() -> Option<&'a mut C>).take(length)),
        }
    }

    fn assert_claim(buffer: &mut AssertionBuffer) {
        buffer.claim_mutable::<C>();
    }
}

impl<'a> ViewSeal<'a> for entity::Identifier {
    type Result = iter::Copied<slice::Iter<'a, Self>>;

    unsafe fn view(
        _columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        _component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result {
        unsafe { slice::from_raw_parts_mut::<'a, Self>(entity_identifiers.0, length) }
            .iter()
            .copied()
    }

    fn assert_claim(_buffer: &mut AssertionBuffer) {}
}

pub trait ViewsSeal<'a>: Claim {
    type Results: Iterator;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Results;

    fn assert_claims(buffer: &mut AssertionBuffer);
}

impl<'a> ViewsSeal<'a> for Null {
    type Results = iter::Repeat<result::Null>;

    unsafe fn view(
        _columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut entity::Identifier, usize),
        _length: usize,
        _component_map: &HashMap<TypeId, usize>,
    ) -> Self::Results {
        iter::repeat(result::Null)
    }

    fn assert_claims(_buffer: &mut AssertionBuffer) {}
}

impl<'a, V, W> ViewsSeal<'a> for (V, W)
where
    V: ViewSeal<'a>,
    W: ViewsSeal<'a>,
{
    type Results = iter::Zip<V::Result, W::Results>;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Results {
        unsafe {
            V::view(columns, entity_identifiers, length, component_map).zip(W::view(
                columns,
                entity_identifiers,
                length,
                component_map,
            ))
        }
    }

    fn assert_claims(buffer: &mut AssertionBuffer) {
        V::assert_claim(buffer);
        W::assert_claims(buffer);
    }
}
