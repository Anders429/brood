mod storage;

use crate::{
    component::Component,
    entity::Null,
};
use storage::Storage;

pub trait Seal: Storage {}

impl Seal for Null {}

impl<C, E> Seal for (C, E)
where
    C: Component,
    E: Seal,
{
}
