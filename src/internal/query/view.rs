use crate::{
    component::Component,
    query::{NullViews, Read, View, Write},
};

pub trait ViewSeal {}

impl<C> ViewSeal for Read<C> where C: Component {}

impl<C> ViewSeal for Write<C> where C: Component {}

pub trait ViewsSeal {}

impl ViewsSeal for NullViews {}

impl<'a, V, W> ViewsSeal for (V, W)
where
    V: View<'a>,
    W: ViewsSeal,
{
}
