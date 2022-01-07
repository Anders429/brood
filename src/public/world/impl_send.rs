use crate::{registry::Registry, world::World};

unsafe impl<R> Send for World<R> where R: Registry {}
