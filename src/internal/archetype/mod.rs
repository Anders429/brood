mod impl_debug;
mod impl_drop;
mod impl_eq;
#[cfg(feature = "serde")]
mod impl_serde;

use crate::{
    entities::Entities,
    entity::{Entity, EntityIdentifier},
    internal::entity_allocator::{EntityAllocator, Location},
};
use alloc::vec::Vec;
use core::{
    any::TypeId,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    ptr,
};
use hashbrown::HashMap;

pub(crate) struct Archetype<E>
where
    E: Entity,
{
    entity: PhantomData<E>,

    entity_identifiers: (*mut EntityIdentifier, usize),
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
    pub(crate) fn from_raw_parts(
        entity_identifiers: (*mut EntityIdentifier, usize),
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

        let mut entities_buffer = Vec::with_capacity(size_of::<Vec<()>>() * E::LEN);
        unsafe {
            entities_buffer.set_len(size_of::<Vec<()>>() * E::LEN);
        }

        Self {
            entity: PhantomData,

            entity_identifiers,
            components,
            length,

            component_map,
            offset_map,

            entity_buffer,
            entities_buffer,
        }
    }

    pub(crate) fn new() -> Self {
        let mut entity_identifiers = ManuallyDrop::new(Vec::new());

        let mut components = Vec::new();
        for _ in 0..E::LEN {
            let mut v = ManuallyDrop::new(Vec::new());
            components.push((v.as_mut_ptr(), v.capacity()));
        }

        Self::from_raw_parts(
            (
                entity_identifiers.as_mut_ptr(),
                entity_identifiers.capacity(),
            ),
            components,
            0,
        )
    }

    pub(crate) unsafe fn push<F>(
        &mut self,
        entity: F,
        entity_allocator: &mut EntityAllocator,
        key: ptr::NonNull<u8>,
    ) where
        F: Entity,
    {
        // Load the components of `entity` into the buffer.
        entity.into_buffer(self.entity_buffer.as_mut_ptr(), &self.offset_map);

        E::push_components_from_buffer(
            self.entity_buffer.as_ptr(),
            &mut self.components,
            self.length,
        );

        let mut entity_identifiers = ManuallyDrop::new(Vec::from_raw_parts(
            self.entity_identifiers.0,
            self.length,
            self.entity_identifiers.1,
        ));
        entity_identifiers.push(entity_allocator.allocate(Location {
            key,
            index: self.length,
        }));
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );

        self.length += 1;
    }

    pub(crate) unsafe fn extend<F>(
        &mut self,
        entities: F,
        entity_allocator: &mut EntityAllocator,
        key: ptr::NonNull<u8>,
    ) where
        F: Entities,
    {
        let component_len = entities.component_len();

        // Load the component `Vec`s of `entities` into the buffer.
        entities.into_buffer(self.entities_buffer.as_mut_ptr(), &self.component_map);

        // Push the components all at once.
        E::extend_components_from_buffer(
            self.entities_buffer.as_ptr(),
            &mut self.components,
            self.length,
        );

        let mut entity_identifiers_v = ManuallyDrop::new(Vec::from_raw_parts(
            self.entity_identifiers.0,
            self.length,
            self.entity_identifiers.1,
        ));
        entity_identifiers_v.extend(entity_allocator.allocate_batch(
            (self.length..(self.length + component_len)).map(|index| Location { key, index }),
        ));
        self.entity_identifiers = (
            entity_identifiers_v.as_mut_ptr(),
            entity_identifiers_v.capacity(),
        );

        self.length += component_len;
    }
}

#[cfg(test)]
mod tests {
    use super::Archetype;
    use crate::{
        entities, entity,
        entity::{EntityIdentifier, NullEntity},
    };
    use alloc::{borrow::ToOwned, string::String, vec};

    #[test]
    fn push() {
        let mut archetype = Archetype::<(usize, (bool, NullEntity))>::new();

        unsafe {
            archetype.push(entity!(1_usize, false), EntityIdentifier::new(0, 0));
            archetype.push(
                entity!(1_usize, false, 2_usize),
                EntityIdentifier::new(1, 0),
            );
            archetype.push(entity!(false, 3_usize), EntityIdentifier::new(2, 0));
            archetype.push(
                entity!(1_usize, false, 2_usize),
                EntityIdentifier::new(3, 0),
            );
            archetype.push(entity!(false, 3_usize), EntityIdentifier::new(4, 0));
        }
    }

    #[test]
    fn push_string() {
        let mut archetype = Archetype::<(String, NullEntity)>::new();

        unsafe {
            archetype.push(entity!("foo".to_owned()), EntityIdentifier::new(0, 0));
        }
    }

    #[test]
    fn extend() {
        let mut archetype = Archetype::<(usize, (bool, NullEntity))>::new();

        unsafe {
            archetype.extend(
                entities!(@internal (1_usize, false); 100),
                vec![EntityIdentifier::new(0, 0); 100].into_iter(),
            );
        }
    }
}
