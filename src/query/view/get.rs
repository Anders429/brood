use crate::{
    entity,
    query::view::{View, Views},
};

/// Type marker for the location of a view.
pub enum Index {}

pub trait Get<'a, T, I> {
    type Remainder: Views<'a>;
}

impl<'a, T, V> Get<'a, &'a T, Index> for (&'a T, V)
where
    V: Views<'a>,
{
    type Remainder = V;
}

impl<'a, T, V> Get<'a, &'a mut T, Index> for (&'a mut T, V)
where
    V: Views<'a>,
{
    type Remainder = V;
}

impl<'a, T, V> Get<'a, Option<&'a T>, Index> for (Option<&'a T>, V)
where
    V: Views<'a>,
{
    type Remainder = V;
}

impl<'a, T, V> Get<'a, Option<&'a mut T>, Index> for (Option<&'a mut T>, V)
where
    V: Views<'a>,
{
    type Remainder = V;
}

impl<'a, V> Get<'a, entity::Identifier, Index> for (entity::Identifier, V)
where
    V: Views<'a>,
{
    type Remainder = V;
}

impl<'a, I, T, V, W> Get<'a, T, (I,)> for (V, W)
where
    V: View<'a>,
    W: Get<'a, T, I>,
{
    type Remainder = (V, <W as Get<'a, T, I>>::Remainder);
}
