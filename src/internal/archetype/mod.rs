mod identifier;
mod impl_debug;
mod impl_drop;
mod impl_eq;
#[cfg(feature = "serde")]
mod impl_serde;

pub(crate) use identifier::{Identifier, IdentifierBuffer, IdentifierIterator};

use crate::{
    component::Component,
    entities::{Entities, EntitiesIter},
    entity::{Entity, EntityIdentifier},
    internal::entity_allocator::{EntityAllocator, Location},
    query::Views,
    registry::Registry,
};
use alloc::vec::Vec;
use core::{
    any::TypeId,
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    slice,
};
use hashbrown::HashMap;

pub(crate) struct Archetype<R>
where
    R: Registry,
{
    identifier_buffer: IdentifierBuffer<R>,

    entity_identifiers: (*mut EntityIdentifier, usize),
    components: Vec<(*mut u8, usize)>,
    length: usize,

    component_map: HashMap<TypeId, usize>,
}

impl<R> Archetype<R>
where
    R: Registry,
{
    pub(crate) unsafe fn from_raw_parts(
        identifier_buffer: IdentifierBuffer<R>,
        entity_identifiers: (*mut EntityIdentifier, usize),
        components: Vec<(*mut u8, usize)>,
        length: usize,
    ) -> Self {
        let mut component_map = HashMap::new();
        R::create_component_map_for_key(&mut component_map, 0, identifier_buffer.iter());

        Self {
            identifier_buffer,

            entity_identifiers,
            components,
            length,

            component_map,
        }
    }

    pub(crate) unsafe fn new(identifier_buffer: IdentifierBuffer<R>) -> Self {
        let mut entity_identifiers = ManuallyDrop::new(Vec::new());

        let entity_len = identifier_buffer.iter().filter(|b| *b).count();
        let mut components = Vec::with_capacity(entity_len);
        for _ in 0..entity_len {
            let mut v = ManuallyDrop::new(Vec::new());
            components.push((v.as_mut_ptr(), v.capacity()));
        }

        Self::from_raw_parts(
            identifier_buffer,
            (
                entity_identifiers.as_mut_ptr(),
                entity_identifiers.capacity(),
            ),
            components,
            0,
        )
    }

    pub(crate) unsafe fn push<E>(
        &mut self,
        entity: E,
        entity_allocator: &mut EntityAllocator<R>,
    ) -> EntityIdentifier
    where
        E: Entity,
    {
        entity.push_components(&self.component_map, &mut self.components, self.length);

        let entity_identifier = entity_allocator.allocate(Location {
            identifier: self.identifier_buffer.as_identifier(),
            index: self.length,
        });

        let mut entity_identifiers = ManuallyDrop::new(Vec::from_raw_parts(
            self.entity_identifiers.0,
            self.length,
            self.entity_identifiers.1,
        ));
        entity_identifiers.push(entity_identifier);
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );

        self.length += 1;

        entity_identifier
    }

    pub(crate) unsafe fn extend<E>(
        &mut self,
        entities: EntitiesIter<E>,
        entity_allocator: &mut EntityAllocator<R>,
    ) -> impl Iterator<Item = EntityIdentifier>
    where
        E: Entities,
    {
        let component_len = entities.entities.component_len();

        entities
            .entities
            .extend_components(&self.component_map, &mut self.components, self.length);

        let entity_identifiers = entity_allocator.allocate_batch(
            (self.length..(self.length + component_len)).map(|index| Location {
                identifier: self.identifier_buffer.as_identifier(),
                index,
            }),
        );

        let mut entity_identifiers_v = ManuallyDrop::new(Vec::from_raw_parts(
            self.entity_identifiers.0,
            self.length,
            self.entity_identifiers.1,
        ));
        entity_identifiers_v.extend(entity_identifiers.iter());
        self.entity_identifiers = (
            entity_identifiers_v.as_mut_ptr(),
            entity_identifiers_v.capacity(),
        );

        self.length += component_len;

        entity_identifiers.into_iter()
    }

    pub(crate) fn view<'a, V>(&mut self) -> V::Results
    where
        V: Views<'a>,
    {
        unsafe {
            V::view(
                &self.components,
                self.entity_identifiers,
                self.length,
                &self.component_map,
            )
        }
    }

    pub(crate) unsafe fn set_component_unchecked<C>(&mut self, index: usize, component: C)
    where
        C: Component,
    {
        *slice::from_raw_parts_mut(
            self.components
                .get_unchecked(
                    *self
                        .component_map
                        .get(&TypeId::of::<C>())
                        .unwrap_unchecked(),
                )
                .0 as *mut C,
            self.length,
        )
        .get_unchecked_mut(index) = component;
    }

    pub(crate) unsafe fn remove_row_unchecked(
        &mut self,
        index: usize,
        entity_allocator: &mut EntityAllocator<R>,
    ) {
        R::remove_component_row(
            index,
            &self.components,
            self.length,
            self.identifier_buffer.iter(),
        );

        let mut entity_identifiers = ManuallyDrop::new(Vec::from_raw_parts(
            self.entity_identifiers.0,
            self.length,
            self.entity_identifiers.1,
        ));
        // Update swapped index if this isn't the last row.
        if index < self.length - 1 {
            entity_allocator.modify_location_index_unchecked(
                *entity_identifiers.last().unwrap_unchecked(),
                index,
            );
        }
        entity_identifiers.swap_remove(index);

        self.length -= 1;
    }

    pub(crate) unsafe fn pop_row_unchecked(
        &mut self,
        index: usize,
        entity_allocator: &mut EntityAllocator<R>,
    ) -> (EntityIdentifier, Vec<u8>) {
        let size_of_components = self.identifier_buffer.size_of_components();
        let mut bytes = Vec::with_capacity(size_of_components);
        R::pop_component_row(
            index,
            bytes.as_mut_ptr(),
            &self.components,
            self.length,
            self.identifier_buffer.iter(),
        );
        bytes.set_len(size_of_components);

        let mut entity_identifiers = ManuallyDrop::new(Vec::from_raw_parts(
            self.entity_identifiers.0,
            self.length,
            self.entity_identifiers.1,
        ));
        // Update swapped index if this isn't the last row.
        if index < self.length - 1 {
            entity_allocator.modify_location_index_unchecked(
                *entity_identifiers.last().unwrap_unchecked(),
                index,
            );
        }
        let entity_identifier = entity_identifiers.swap_remove(index);

        self.length -= 1;

        (entity_identifier, bytes)
    }

    pub(crate) unsafe fn push_from_buffer_and_component<C>(
        &mut self,
        entity_identifier: EntityIdentifier,
        buffer: Vec<u8>,
        component: C,
    ) -> usize
    where
        C: Component,
    {
        R::push_components_from_buffer_and_component(
            buffer.as_ptr(),
            MaybeUninit::new(component),
            &mut self.components,
            self.length,
            self.identifier_buffer.iter(),
        );

        let mut entity_identifiers = ManuallyDrop::new(Vec::from_raw_parts(
            self.entity_identifiers.0,
            self.length,
            self.entity_identifiers.1,
        ));
        entity_identifiers.push(entity_identifier);
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );

        self.length += 1;

        self.length - 1
    }

    pub(crate) unsafe fn push_from_buffer_skipping_component<C>(
        &mut self,
        entity_identifier: EntityIdentifier,
        buffer: Vec<u8>,
    ) -> usize
    where
        C: Component,
    {
        R::push_components_from_buffer_skipping_component(
            buffer.as_ptr(),
            PhantomData::<C>,
            &mut self.components,
            self.length,
            self.identifier_buffer.iter(),
        );

        let mut entity_identifiers = ManuallyDrop::new(Vec::from_raw_parts(
            self.entity_identifiers.0,
            self.length,
            self.entity_identifiers.1,
        ));
        entity_identifiers.push(entity_identifier);
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );

        self.length += 1;

        self.length - 1
    }

    pub(crate) unsafe fn identifier(&self) -> Identifier<R> {
        self.identifier_buffer.as_identifier()
    }

    pub(crate) fn entity_identifiers(&self) -> impl Iterator<Item = &EntityIdentifier> {
        unsafe { slice::from_raw_parts(self.entity_identifiers.0, self.length) }.iter()
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
