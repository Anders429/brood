use crate::{registry::RegistrySync, world::World};

unsafe impl<R> Sync for World<R> where R: RegistrySync {}
