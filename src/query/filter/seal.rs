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

pub trait Seal {}

impl Seal for None {}

impl<C> Seal for Has<C> where C: Component {}

impl<F> Seal for Not<F> where F: Filter {}

impl<F1, F2> Seal for And<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
}

impl<F1, F2> Seal for Or<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
}

impl<C> Seal for &C where C: Component {}

impl<C> Seal for &mut C where C: Component {}

impl<C> Seal for Option<&C> where C: Component {}

impl<C> Seal for Option<&mut C> where C: Component {}

impl Seal for entity::Identifier {}

impl Seal for view::Null {}

impl<V, W> Seal for (V, W)
where
    V: Filter,
    W: Filter,
{
}
