use super::World;
use crate::{entity::NullEntity, internal::registry::RegistryDebug};
use core::{
    fmt::{self, Debug},
    marker::PhantomData,
};

impl<R> Debug for World<R>
where
    R: RegistryDebug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_map = f.debug_map();
        for (key, archetype) in &self.archetypes {
            unsafe {
                R::debug::<NullEntity>(
                    &key,
                    0,
                    0,
                    &archetype,
                    &mut debug_map,
                    PhantomData,
                )
            }
        }
        debug_map.finish()
    }
}
