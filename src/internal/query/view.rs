use crate::{
    component::Component,
    entity::EntityIdentifier,
    query::{NullResult, NullViews},
};
use core::{any::TypeId, iter, slice};
use hashbrown::HashMap;

pub trait ViewSeal<'a> {
    type Result: Iterator;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut EntityIdentifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result;
}

impl<'a, C> ViewSeal<'a> for &C where C: Component {
    type Result = slice::Iter<'a, C>;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut EntityIdentifier, usize),
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
        _entity_identifiers: (*mut EntityIdentifier, usize),
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

impl<'a> ViewSeal<'a> for EntityIdentifier {
    type Result = iter::Cloned<slice::Iter<'a, Self>>;

    unsafe fn view(
        _columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut EntityIdentifier, usize),
        length: usize,
        _component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result {
        slice::from_raw_parts_mut::<'a, Self>(entity_identifiers.0, length)
            .iter()
            .cloned()
    }
}

pub trait ViewsSeal<'a> {
    type Results: Iterator;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        entity_identifiers: (*mut EntityIdentifier, usize),
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Results;
}

impl<'a> ViewsSeal<'a> for NullViews {
    type Results = iter::Repeat<NullResult>;

    unsafe fn view(
        _columns: &[(*mut u8, usize)],
        _entity_identifiers: (*mut EntityIdentifier, usize),
        _length: usize,
        _component_map: &HashMap<TypeId, usize>,
    ) -> Self::Results {
        iter::repeat(NullResult)
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
        entity_identifiers: (*mut EntityIdentifier, usize),
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
