mod into_raw;
mod length;
mod storage;

use crate::{component::Component, entities::Null};
use alloc::vec::Vec;
use into_raw::IntoRaw;
use length::Length;
use storage::Storage;

pub trait Seal: IntoRaw + Length + Storage {}

impl Seal for Null {}

impl<C, E> Seal for (Vec<C>, E)
where
    C: Component,
    E: Seal,
{
}
