use crate::{
    component::Component,
    query::{NullResult, NullViews, Read, Write},
};
use core::{any::TypeId, iter, slice};
use hashbrown::HashMap;

pub trait ViewSeal<'a> {
    type Result: Iterator;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Result;
}

impl<'a, C> ViewSeal<'a> for Read<C>
where
    C: Component,
{
    type Result = slice::Iter<'a, C>;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
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

impl<'a, C> ViewSeal<'a> for Write<C>
where
    C: Component,
{
    type Result = slice::IterMut<'a, C>;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
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

pub trait ViewsSeal<'a> {
    type Results: Iterator;

    unsafe fn view(
        columns: &[(*mut u8, usize)],
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Results;
}

impl<'a> ViewsSeal<'a> for NullViews {
    type Results = iter::Repeat<NullResult>;

    unsafe fn view(
        _columns: &[(*mut u8, usize)],
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
        length: usize,
        component_map: &HashMap<TypeId, usize>,
    ) -> Self::Results {
        V::view(columns, length, component_map).zip(W::view(columns, length, component_map))
    }
}
