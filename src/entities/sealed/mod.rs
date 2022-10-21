mod contains;
mod length;
mod storage;

pub(crate) use contains::Contains;

use crate::{
    component::Component,
    entities::Null,
};
use alloc::vec::Vec;
use length::Length;
use storage::Storage;

pub trait Sealed: Contains + Length + Storage {}

impl Sealed for Null {}

impl<C, E> Sealed for (Vec<C>, E)
where
    C: Component,
    E: Sealed,
{
}
