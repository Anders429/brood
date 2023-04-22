use crate::{
    registry,
    world::World,
};

// SAFETY: This type is safe to send between threads, since all pointers are owned and cannot be
// mutated without mutable access.
unsafe impl<Registry, Resources> Send for World<Registry, Resources>
where
    Registry: registry::Registry + Send,
    Resources: Send,
{
}
