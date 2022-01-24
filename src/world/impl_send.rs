use crate::{registry::RegistrySend, world::World};

unsafe impl<R> Send for World<R> where R: RegistrySend {}
