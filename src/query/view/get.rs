use crate::entity;

/// Type marker for the location of a view.
pub enum Index {}

pub trait Get<'a, T, I> {
    type Remainder;
}

impl<'a, T, V> Get<'a, &'a T, Index> for (&'a T, V) {
    type Remainder = V;
}

impl<'a, T, V> Get<'a, &'a mut T, Index> for (&'a mut T, V) {
    type Remainder = V;
}

impl<'a, T, V> Get<'a, Option<&'a T>, Index> for (Option<&'a T>, V) {
    type Remainder = V;
}

impl<'a, T, V> Get<'a, Option<&'a mut T>, Index> for (Option<&'a mut T>, V) {
    type Remainder = V;
}

impl<V> Get<'_, entity::Identifier, Index> for (entity::Identifier, V) {
    type Remainder = V;
}

impl<'a, I, T, V, W> Get<'a, T, (I,)> for (V, W)
where
    W: Get<'a, T, I>,
{
    type Remainder = (V, <W as Get<'a, T, I>>::Remainder);
}
