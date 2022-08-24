use crate::{component::Component, entities, entities::Entities, entity::Null};
use alloc::vec::Vec;

pub trait Unzip: Sized {
    type Entities: Entities + Default + Extend<Self>;

    fn unzip<T>(entities: T) -> Self::Entities
    where
        T: IntoIterator<Item = Self>;
}

impl Unzip for Null {
    type Entities = entities::Null;

    fn unzip<T>(_entities: T) -> Self::Entities
    where
        T: IntoIterator<Item = Self>,
    {
        entities::Null
    }
}

impl<C, E> Unzip for (C, E)
where
    C: Component,
    E: Unzip,
{
    type Entities = (Vec<C>, E::Entities);

    fn unzip<T>(entities: T) -> Self::Entities
    where
        T: IntoIterator<Item = Self>,
    {
        entities.into_iter().unzip()
    }
}
