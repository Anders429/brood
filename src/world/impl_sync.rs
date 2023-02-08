use crate::{
    registry::Registry,
    world::World,
};

// SAFETY: This type is safe to share between multiple threads as you can't mutate it without a
// &mut reference.
unsafe impl<R, Resources> Sync for World<R, Resources>
where
    R: Registry + Sync,
    Resources: Sync,
{
}
