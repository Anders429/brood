use crate::{
    component::Component,
    entity,
    internal::query::claim::Claim,
    query::{result, view::Null},
};
use core::{any::TypeId, iter, slice};
use either::Either;
use hashbrown::HashMap;

pub trait ViewSeal<'a>: Claim {
    type Result: Iterator;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result;
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
        slice::from_raw_parts::<'a, C>(
            columns
                .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                .0 as *mut C,
            length,
        )
        .iter()
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
        slice::from_raw_parts_mut::<'a, C>(
            columns
                .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                .0 as *mut C,
            length,
        )
        .iter_mut()
    }
}

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
                slice::from_raw_parts(columns.get_unchecked(*index).0 as *mut C, length)
                    .iter()
                    .map(wrap_some),
            ),
            None => Either::Left(iter::repeat(None).take(length)),
        }
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
                slice::from_raw_parts_mut(columns.get_unchecked(*index).0 as *mut C, length)
                    .iter_mut()
                    .map(wrap_some),
            ),
            None => Either::Left(iter::repeat_with(none as fn() -> Option<&'a mut C>).take(length)),
        }
    }
}

impl<'a> ViewSeal<'a> for entity::Identifier {
    type Result = iter::Cloned<slice::Iter<'a, Self>>;

    unsafe fn view(
        _columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        _component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result {
        slice::from_raw_parts_mut::<'a, Self>(entity_identifiers.0, length)
            .iter()
            .cloned()
    }
}

pub trait ViewsSeal<'a>: Claim {
    type Results: Iterator;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut entity::Identifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Results;
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
        V::view(columns, entity_identifiers, length, component_map).zip(W::view(
            columns,
            entity_identifiers,
            length,
            component_map,
        ))
    }
}
