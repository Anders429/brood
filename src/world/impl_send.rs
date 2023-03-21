use crate::{
    registry::Registry,
    world::World,
};

// SAFETY: This type is safe to send between threads, since all pointers are owned and cannot be
// mutated without mutable access.
unsafe impl<R, Resources> Send for World<R, Resources>
where
    R: Registry + Send,
    Resources: Send,
{
}
