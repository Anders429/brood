use crate::{
    registry::RegistrySync,
    world::World,
};

// SAFETY: This type is safe to share between multiple threads as you can't mutate it without a
// &mut reference.
unsafe impl<R> Sync for World<R> where R: RegistrySync {}
