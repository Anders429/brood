mod impl_debug;
mod impl_drop;
mod impl_eq;
mod impl_send;
#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod impl_serde;

pub(crate) mod identifier;

pub(crate) use identifier::{Identifier, IdentifierRef};
#[cfg(feature = "serde")]
pub(crate) use impl_serde::{DeserializeColumn, SerializeColumn};

#[cfg(feature = "parallel")]
use crate::query::view::ParViews;
use crate::{
    component::Component,
    entities,
    entities::Entities,
    entity,
    entity::{
        allocator::{Location, Locations},
        Entity,
    },
    query::view::Views,
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
    identifier_buffer: Identifier<R>,

    entity_identifiers: (*mut entity::Identifier, usize),
    components: Vec<(*mut u8, usize)>,
    length: usize,

    component_map: HashMap<TypeId, usize>,
}

impl<R> Archetype<R>
where
    R: Registry,
{
    pub(crate) unsafe fn from_raw_parts(
        identifier_buffer: Identifier<R>,
        entity_identifiers: (*mut entity::Identifier, usize),
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

    pub(crate) unsafe fn new(identifier_buffer: Identifier<R>) -> Self {
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
        entity_allocator: &mut entity::Allocator<R>,
    ) -> entity::Identifier
    where
        E: Entity,
    {
        entity.push_components(&self.component_map, &mut self.components, self.length);

        let entity_identifier = entity_allocator.allocate(Location {
            identifier: self.identifier_buffer.as_ref(),
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
        entities: entities::Batch<E>,
        entity_allocator: &mut entity::Allocator<R>,
    ) -> Vec<entity::Identifier>
    where
        E: Entities,
    {
        let component_len = entities.entities.component_len();

        entities
            .entities
            .extend_components(&self.component_map, &mut self.components, self.length);

        let entity_identifiers = entity_allocator.allocate_batch(Locations::new(
            self.length..(self.length + component_len),
            self.identifier_buffer.as_ref(),
        ));

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

        entity_identifiers
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

    #[cfg(feature = "parallel")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "parallel")))]
    pub(crate) fn par_view<'a, V>(&mut self) -> V::ParResults
    where
        V: ParViews<'a>,
    {
        unsafe {
            V::par_view(
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
                .0
                .cast::<C>(),
            self.length,
        )
        .get_unchecked_mut(index) = component;
    }

    pub(crate) unsafe fn remove_row_unchecked(
        &mut self,
        index: usize,
        entity_allocator: &mut entity::Allocator<R>,
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
        entity_allocator: &mut entity::Allocator<R>,
    ) -> (entity::Identifier, Vec<u8>) {
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
        entity_identifier: entity::Identifier,
        buffer: *const u8,
        component: C,
    ) -> usize
    where
        C: Component,
    {
        R::push_components_from_buffer_and_component(
            buffer,
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
        entity_identifier: entity::Identifier,
        buffer: *const u8,
    ) -> usize
    where
        C: Component,
    {
        R::push_components_from_buffer_skipping_component(
            buffer,
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

    pub(crate) unsafe fn identifier(&self) -> IdentifierRef<R> {
        self.identifier_buffer.as_ref()
    }

    #[cfg(feature = "serde")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
    pub(crate) fn entity_identifiers(&self) -> impl Iterator<Item = &entity::Identifier> {
        unsafe { slice::from_raw_parts(self.entity_identifiers.0, self.length) }.iter()
    }
}
