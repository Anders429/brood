mod assertions;
mod length;
mod storage;

use crate::{component::Component, registry::Null};
use assertions::Assertions;
use length::Length;
use storage::Storage;

pub trait Seal: Assertions + Length + Storage {}

impl Seal for Null {}

impl<C, R> Seal for (C, R)
where
    C: Component,
    R: Seal,
{
}
