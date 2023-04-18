use crate::{
    component::Component,
    entity,
    query::{
        filter::{
            And,
            Filter,
            Has,
            None,
            Not,
            Or,
        },
        view,
    },
};

pub trait Sealed {}

impl Sealed for None {}

impl<C> Sealed for Has<C> {}

impl<F> Sealed for Not<F> where F: Filter {}

impl<F1, F2> Sealed for And<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
}

impl<F1, F2> Sealed for Or<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
}

impl<C> Sealed for &C where C: Component {}

impl<C> Sealed for &mut C where C: Component {}

impl<C> Sealed for Option<&C> where C: Component {}

impl<C> Sealed for Option<&mut C> where C: Component {}

impl Sealed for entity::Identifier {}

impl Sealed for view::Null {}

impl<V, W> Sealed for (V, W)
where
    V: Filter,
    W: Filter,
{
}
