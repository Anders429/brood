use crate::component::Component;
use core::{iter, marker::PhantomData};
use alloc::vec;

pub trait View<'a> {
    type Item: 'a;
}

pub struct Read<C> where C: Component {
    component: PhantomData<C>,
}

impl<'a, C> View<'a> for Read<C> where C: Component {
    type Item = &'a C;
}

pub struct Write<C> where C: Component {
    component: PhantomData<C>,
}

impl<'a, C> View<'a> for Write<C> where C: Component {
    type Item = &'a mut C;
}

pub struct NullViews;

pub trait Views<'a> {
    type Results;
}

impl<'a> Views<'a> for NullViews {
    type Results = (); 
}

impl<'a, V, W> Views<'a> for (V, W) where V: View<'a>, W: Views<'a> {
    type Results = (iter::Flatten<vec::IntoIter<&'a [V::Item]>>, W::Results);
}
