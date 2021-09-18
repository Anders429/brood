mod impl_debug;
mod impl_drop;
mod impl_eq;
#[cfg(feature = "serde")]
mod impl_serde;

use crate::{
    entity::{Entities, Entity},
};
use alloc::vec::Vec;
use core::{any::TypeId, marker::PhantomData, mem::ManuallyDrop};
use hashbrown::HashMap;

pub(crate) struct Archetype<E>
where
    E: Entity,
{
    entity: PhantomData<E>,

    components: Vec<(*mut u8, usize)>,
    length: usize,

    component_map: HashMap<TypeId, usize>,
    offset_map: HashMap<TypeId, isize>,

    entity_buffer: Vec<u8>,
    entities_buffer: Vec<u8>,
}

impl<E> Archetype<E>
where
    E: Entity,
{
    pub(crate) fn from_components_and_length(
        components: Vec<(*mut u8, usize)>,
        length: usize,
    ) -> Self {
        let mut component_map = HashMap::new();
        E::create_component_map(&mut component_map, 0);

        let mut offset_map = HashMap::new();
        E::create_offset_map(&mut offset_map, 0);

        let mut entity_buffer = Vec::with_capacity(E::BYTE_LEN);
        unsafe {
            entity_buffer.set_len(E::BYTE_LEN);
        }

        let mut entities_buffer = Vec::with_capacity(24 * E::LEN);
        unsafe {
            entities_buffer.set_len(24 * E::LEN);
        }

        Self {
            entity: PhantomData,

            components,
            length,

            component_map,
            offset_map,

            entity_buffer,
            entities_buffer,
        }
    }

    pub(crate) fn new() -> Self {
        let mut components = Vec::new();
        for _ in 0..E::LEN {
            let mut v = ManuallyDrop::new(Vec::new());
            components.push((v.as_mut_ptr(), v.capacity()));
        }

        Self::from_components_and_length(components, 0)
    }

    pub(crate) unsafe fn push<F>(&mut self, entity: F)
    where
        F: Entity,
    {
        // Load the components of `entity` into the buffer.
        unsafe {
            entity.into_buffer(self.entity_buffer.as_mut_ptr(), &self.offset_map);
        }

        E::push_components_from_buffer(
            self.entity_buffer.as_ptr(),
            &mut self.components,
            self.length,
        );

        self.length += 1;
    }

    pub(crate) unsafe fn extend<F>(&mut self, entities: F)
    where
        F: Entities,
    {
        let component_len = entities.component_len();

        // Load the component `Vec`s of `entities` into the buffer.
        unsafe {
            entities.into_buffer(self.entities_buffer.as_mut_ptr(), &self.component_map);
        }

        // Push the components all at once.
        E::extend_components_from_buffer(
            self.entities_buffer.as_ptr(),
            &mut self.components,
            self.length,
        );

        self.length += component_len;
    }
}

#[cfg(test)]
mod tests {
    use super::Archetype;
    use crate::{entities, entity, entity::NullEntity};
    use alloc::{borrow::ToOwned, string::String};

    #[test]
    fn push() {
        let mut archetype = Archetype::<(usize, (bool, NullEntity))>::new();

        unsafe {
            archetype.push(entity!(1_usize, false));
            archetype.push(entity!(1_usize, false, 2_usize));
            archetype.push(entity!(false, 3_usize));
            archetype.push(entity!(1_usize, false, 2_usize));
            archetype.push(entity!(false, 3_usize));
        }
    }

    #[test]
    fn push_string() {
        let mut archetype = Archetype::<(String, NullEntity)>::new();

        unsafe {
            archetype.push(entity!("foo".to_owned()));
        }
    }

    #[test]
    fn extend() {
        let mut archetype = Archetype::<(usize, (bool, NullEntity))>::new();

        unsafe {
            archetype.extend(entities!((1_usize, false); 100));
        }
    }
}
