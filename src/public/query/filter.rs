use crate::component::Component;
use core::marker::PhantomData;

pub trait Filter {}

pub struct Has<C>
where
    C: Component,
{
    component: PhantomData<C>,
}

impl<C> Filter for Has<C> where C: Component {}

pub struct Not<F>
where
    F: Filter,
{
    filter: PhantomData<F>,
}

impl<F> Filter for Not<F> where F: Filter {}

pub struct And<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    filter_1: PhantomData<F1>,
    filter_2: PhantomData<F2>,
}

impl<F1, F2> Filter for And<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
}

pub struct Or<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    filter_1: PhantomData<F1>,
    filter_2: PhantomData<F2>,
}

impl<F1, F2> Filter for Or<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
}
