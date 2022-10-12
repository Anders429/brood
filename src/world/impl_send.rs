use crate::{
    registry::RegistrySend,
    world::World,
};

// SAFETY: This type is safe to send between threads, since all pointers are owned and cannot be
// mutated without mutable access.
unsafe impl<R> Send for World<R> where R: RegistrySend {}
