mod storage;
mod unzip;

use crate::{component::Component, entity::Null};
use storage::Storage;
use unzip::Unzip;

pub trait Seal: Storage + Unzip {}

impl Seal for Null {}

impl<C, E> Seal for (C, E)
where
    C: Component,
    E: Seal,
{
}
