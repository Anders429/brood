use crate::{
    registry,
    world::World,
};

// SAFETY: This type is safe to share between multiple threads as you can't mutate it without a
// &mut reference.
unsafe impl<Registry, Resources> Sync for World<Registry, Resources>
where
    Registry: registry::Registry + Sync,
    Resources: Sync,
{
}
