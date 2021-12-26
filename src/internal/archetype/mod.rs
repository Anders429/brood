mod identifier;
mod impl_debug;
mod impl_drop;
mod impl_eq;
#[cfg(feature = "serde")]
mod impl_serde;

pub(crate) use identifier::{Identifier, IdentifierBuffer, IdentifierIter};

use crate::{
    entities::{Entities, EntitiesIter},
    entity::{Entity, EntityIdentifier},
    internal::entity_allocator::{EntityAllocator, Location},
    query::Views,
    registry::Registry,
};
use alloc::vec::Vec;
use core::{any::TypeId, mem::ManuallyDrop, slice};
use hashbrown::HashMap;

pub(crate) struct Archetype<R>
where
    R: Registry,
{
    identifier: IdentifierBuffer<R>,

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
        identifier: IdentifierBuffer<R>,
        entity_identifiers: (*mut EntityIdentifier, usize),
        components: Vec<(*mut u8, usize)>,
        length: usize,
    ) -> Self {
        let mut component_map = HashMap::new();
        R::create_component_map_for_key(
            &mut component_map,
            0,
            identifier.iter(),
        );

        Self {
            identifier,

            entity_identifiers,
            components,
            length,

            component_map,
        }
    }

    pub(crate) unsafe fn new(identifier: IdentifierBuffer<R>) -> Self {
        let mut entity_identifiers = ManuallyDrop::new(Vec::new());

        let entity_len = identifier.iter().filter(|b| *b).count();
        let mut components = Vec::with_capacity(entity_len);
        for _ in 0..entity_len {
            let mut v = ManuallyDrop::new(Vec::new());
            components.push((v.as_mut_ptr(), v.capacity()));
        }

        Self::from_raw_parts(
            identifier,
            (
                entity_identifiers.as_mut_ptr(),
                entity_identifiers.capacity(),
            ),
            components,
            0,
        )
    }

    pub(crate) unsafe fn push<E>(&mut self, entity: E, entity_allocator: &mut EntityAllocator<R>)
    where
        E: Entity,
    {
        entity.push_components(&self.component_map, &mut self.components, self.length);

        let mut entity_identifiers = ManuallyDrop::new(Vec::from_raw_parts(
            self.entity_identifiers.0,
            self.length,
            self.entity_identifiers.1,
        ));
        entity_identifiers.push(entity_allocator.allocate(Location {
            identifier: self.identifier.as_identifier(),
            index: self.length,
        }));
        self.entity_identifiers = (
            entity_identifiers.as_mut_ptr(),
            entity_identifiers.capacity(),
        );

        self.length += 1;
    }

    pub(crate) unsafe fn extend<E>(
        &mut self,
        entities: EntitiesIter<E>,
        entity_allocator: &mut EntityAllocator<R>,
    ) where
        E: Entities,
    {
        let component_len = entities.entities.component_len();

        entities
            .entities
            .extend_components(&self.component_map, &mut self.components, self.length);

        let mut entity_identifiers_v = ManuallyDrop::new(Vec::from_raw_parts(
            self.entity_identifiers.0,
            self.length,
            self.entity_identifiers.1,
        ));
        entity_identifiers_v.extend(entity_allocator.allocate_batch(
            (self.length..(self.length + component_len)).map(|index| Location {
                identifier: self.identifier.as_identifier(),
                index,
            }),
        ));
        self.entity_identifiers = (
            entity_identifiers_v.as_mut_ptr(),
            entity_identifiers_v.capacity(),
        );

        self.length += component_len;
    }

    pub(crate) fn view<'a, V>(&mut self) -> V::Results
    where
        V: Views<'a>,
    {
        unsafe { V::view(&self.components, self.length, &self.component_map) }
    }

    pub(crate) fn identifier(&self) -> Identifier<R> {
        self.identifier.as_identifier()
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
