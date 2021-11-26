use crate::component::Component;
use core::marker::PhantomData;

pub trait View<'a> {
    type Item;
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
    type Results = (V::Item, W::Results);
}
