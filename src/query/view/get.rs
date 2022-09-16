use crate::{
    entity,
    query::{
        result::get::Index,
        view::{View, Views},
    },
};

pub trait Get<'a, T, I> {
    type Remainder: Views<'a>;

    fn get(self) -> (T, Self::Remainder);
}

impl<'a, T, V> Get<'a, &'a T, Index> for (&'a T, V)
where
    V: Views<'a>,
{
    type Remainder = V;

    fn get(self) -> (&'a T, Self::Remainder) {
        self
    }
}

impl<'a, T, V> Get<'a, &'a mut T, Index> for (&'a mut T, V)
where
    V: Views<'a>,
{
    type Remainder = V;

    fn get(self) -> (&'a mut T, Self::Remainder) {
        self
    }
}

impl<'a, T, V> Get<'a, Option<&'a T>, Index> for (Option<&'a T>, V)
where
    V: Views<'a>,
{
    type Remainder = V;

    fn get(self) -> (Option<&'a T>, Self::Remainder) {
        self
    }
}

impl<'a, T, V> Get<'a, Option<&'a mut T>, Index> for (Option<&'a mut T>, V)
where
    V: Views<'a>,
{
    type Remainder = V;

    fn get(self) -> (Option<&'a mut T>, Self::Remainder) {
        self
    }
}

impl<'a, V> Get<'a, entity::Identifier, Index> for (entity::Identifier, V)
where
    V: Views<'a>,
{
    type Remainder = V;

    fn get(self) -> (entity::Identifier, Self::Remainder) {
        self
    }
}

impl<'a, I, T, V, W> Get<'a, T, (I,)> for (V, W)
where
    V: View<'a>,
    W: Get<'a, T, I>,
{
    type Remainder = (V, <W as Get<'a, T, I>>::Remainder);

    fn get(self) -> (T, Self::Remainder) {
        let (target, remainder) = self.1.get();
        (target, (self.0, remainder))
    }
}
