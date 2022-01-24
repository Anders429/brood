mod length;
mod storage;

use crate::{component::Component, entities::Null};
use alloc::vec::Vec;
use length::Length;
use storage::Storage;

pub trait Seal: Length + Storage {}

impl Seal for Null {}

impl<C, E> Seal for (Vec<C>, E)
where
    C: Component,
    E: Seal,
{
}
