use crate::{
    entity::{Entities, Entity, NullEntity},
    internal::registry::RegistryDebug,
    registry::Registry,
};
use alloc::boxed::Box;
use core::{
    any::{Any, TypeId},
    fmt::{self, Debug},
    marker::PhantomData,
};
use hashbrown::HashMap;

pub struct World<R>
where
    R: Registry,
    [(); (R::LEN + 7) / 8]: Sized,
{
    registry: PhantomData<R>,
    archetypes: HashMap<[u8; (R::LEN + 7) / 8], Box<dyn Any>>,
    component_map: HashMap<TypeId, usize>,
}

impl<R> World<R>
where
    R: Registry,
    [(); (R::LEN + 7) / 8]: Sized,
{
    pub fn new() -> Self {
        let mut component_map = HashMap::new();
        R::create_component_map(&mut component_map, 0);

        Self {
            registry: PhantomData,
            archetypes: HashMap::new(),
            component_map,
        }
    }

    pub fn push<E>(&mut self, entity: E)
    where
        E: Entity,
    {
        let mut key = [0; (R::LEN + 7) / 8];
        unsafe {
            E::to_key::<R>(&mut key, &self.component_map);
        }

        unsafe {
            R::push::<E, NullEntity, R>(
                entity,
                key,
                &mut self.archetypes,
                0,
                0,
                PhantomData,
                PhantomData,
            );
        }
    }

    pub fn extend<E>(&mut self, entities: E)
    where
        E: Entities,
    {
        let mut key = [0; (R::LEN + 7) / 8];
        unsafe {
            E::to_key::<R>(&mut key, &self.component_map);
        }

        unsafe {
            R::extend::<E, NullEntity, R>(
                entities,
                key,
                &mut self.archetypes,
                0,
                0,
                PhantomData,
                PhantomData,
            );
        }
    }
}

impl<R> Debug for World<R>
where
    R: RegistryDebug,
    [(); (R::LEN + 7) / 8]: Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_map = f.debug_map();
        for (key, archetype) in &self.archetypes {
            unsafe {
                R::debug::<NullEntity, R>(
                    &key,
                    0,
                    0,
                    &archetype,
                    &mut debug_map,
                    PhantomData,
                    PhantomData,
                )
            }
        }
        debug_map.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::World;
    use crate::{entities, entity, registry};
    use alloc::{borrow::ToOwned, string::String};

    #[test]
    fn push() {
        let mut world = World::<registry!(usize, bool, String, ())>::new();

        world.push(entity!(1_usize));
        world.push(entity!(true));
        world.push(entity!("foo".to_owned()));
    }

    #[test]
    fn extend() {
        let mut world = World::<registry!(usize, bool, String, ())>::new();

        world.extend(entities!((1_usize); 100));
        world.extend(entities!((true); 100));
        world.extend(entities!(("foo".to_owned()); 100));
        world.extend(entities!((2_usize, false, "bar".to_owned()); 100));
    }
}
