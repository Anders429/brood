mod storage;

use crate::{
    component::Component,
    entity::Null,
};
use storage::Storage;

pub trait Sealed: Storage {}

impl Sealed for Null {}

impl<C, E> Sealed for (C, E)
where
    C: Component,
    E: Sealed,
{
}
