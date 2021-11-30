use crate::{
    component::Component,
    query::{NullViews, Read, View, Write},
};
use core::{any::TypeId, slice};
use hashbrown::HashMap;

pub trait ViewSeal<'a> {
    type Result: IntoIterator;

    unsafe fn view(
        columns: &[(*const u8, usize)],
        length: usize,
        component_map: &mut HashMap<TypeId, usize>,
    ) -> Self::Result;
}

impl<'a, C> ViewSeal<'a> for Read<C>
where
    C: Component,
{
    type Result = slice::Iter<'a, C>;

    unsafe fn view(
        columns: &[(*const u8, usize)],
        length: usize,
        component_map: &mut HashMap<TypeId, usize>,
    ) -> Self::Result {
        slice::from_raw_parts::<'a, C>(
            columns
                .get_unchecked(*component_map.get(&TypeId::of::<C>()).unwrap_unchecked())
                .0 as *const C,
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
        columns: &[(*const u8, usize)],
        length: usize,
        component_map: &mut HashMap<TypeId, usize>,
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

pub trait ViewsSeal {}

impl ViewsSeal for NullViews {}

impl<'a, V, W> ViewsSeal for (V, W)
where
    V: View<'a>,
    W: ViewsSeal,
{
}
