use crate::component::Component;
use core::marker::PhantomData;

pub trait View {
    // type Result;
}

pub struct Read<C> where C: Component {
    component: PhantomData<C>,
}

impl<C> View for Read<C> where C: Component {
    // type Result<'a> = impl Iterator<Item = &'a C>;
}

pub struct Write<C> where C: Component {
    component: PhantomData<C>,
}

impl<C> View for Write<C> where C: Component {
    // type Result = impl Iterator<Item = &mut C>;
}

pub struct NullViews;

pub trait Views {}

impl Views for NullViews {}

impl<V, W> Views for (V, W) where V: View, W: Views {}
