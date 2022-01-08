use crate::{internal::registry::RegistrySend, world::World};

unsafe impl<R> Send for World<R> where R: RegistrySend {}
