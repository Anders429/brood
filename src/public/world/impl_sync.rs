use crate::{registry::Registry, world::World};

unsafe impl<R> Sync for World<R> where R: Registry {}
