use crate::{
    component::Component,
    internal::{archetype, archetype::Archetype, entity_allocator::Location},
    registry::Registry,
    world::World,
};
use core::any::TypeId;

pub struct Entry<'a, R>
where
    R: Registry,
{
    world: &'a mut World<R>,
    location: Location<R>,
}

impl<'a, R> Entry<'a, R>
where
    R: Registry,
{
    pub(crate) fn new(world: &'a mut World<R>, location: Location<R>) -> Self {
        Self { world, location }
    }

    pub fn add<C>(&mut self, component: C)
    where
        C: Component,
    {
        let component_index = unsafe {
            *self
                .world
                .component_map
                .get(&TypeId::of::<C>())
                .unwrap_unchecked()
        };
        if unsafe { self.location.identifier.get_unchecked(component_index) } {
            // The component already exists within this entity. Replace it.
            unsafe {
                self.world
                    .archetypes
                    .get_mut(&self.location.identifier)
                    .unwrap_unchecked()
                    .set_component_unchecked(self.location.index, component)
            };
        } else {
            // The component needs to be added to the entity.
            let (entity_identifier, current_component_bytes) = unsafe {
                self.world
                    .archetypes
                    .get_mut(&self.location.identifier)
                    .unwrap_unchecked()
                    .pop_row_unchecked(self.location.index, &mut self.world.entity_allocator)
            };
            // Create new identifier buffer.
            let mut raw_identifier_buffer = self.location.identifier.as_vec();
            // Set the component's bit.
            *unsafe { raw_identifier_buffer.get_unchecked_mut(component_index / 8) } |=
                1 << (component_index % 8);
            let identifier_buffer =
                unsafe { archetype::IdentifierBuffer::<R>::new(raw_identifier_buffer) };

            // Insert to the corresponding archetype using the bytes and the new component.
            let archetype_entry = self
                .world
                .archetypes
                .entry(unsafe { identifier_buffer.as_identifier() });
            let archetype_identifier = *archetype_entry.key();
            let index = unsafe {
                archetype_entry
                    .or_insert(Archetype::<R>::new(identifier_buffer))
                    .push_from_buffer_and_component(
                        entity_identifier,
                        current_component_bytes,
                        component,
                    )
            };

            // Update the location.
            unsafe {
                self.world.entity_allocator.modify_location_unchecked(
                    entity_identifier,
                    Location::new(archetype_identifier, index),
                );
            }
        }
    }

    pub fn remove<C>(&mut self)
    where
        C: Component,
    {
        let component_index = unsafe {
            *self
                .world
                .component_map
                .get(&TypeId::of::<C>())
                .unwrap_unchecked()
        };
        if unsafe { self.location.identifier.get_unchecked(component_index) } {
            // The component exists and needs to be removed.
            let (entity_identifier, current_component_bytes) = unsafe {
                self.world
                    .archetypes
                    .get_mut(&self.location.identifier)
                    .unwrap_unchecked()
                    .pop_row_unchecked(self.location.index, &mut self.world.entity_allocator)
            };
            // Create new identifier buffer.
            let mut raw_identifier_buffer = self.location.identifier.as_vec();
            // Unset the component's bit.
            *unsafe { raw_identifier_buffer.get_unchecked_mut(component_index / 8) } ^=
                1 << (component_index % 8);
            let identifier_buffer =
                unsafe { archetype::IdentifierBuffer::<R>::new(raw_identifier_buffer) };

            // Insert to the corresponding archetype using the bytes, skipping the removed
            // component.
            let archetype_entry = self
                .world
                .archetypes
                .entry(unsafe { identifier_buffer.as_identifier() });
            let archetype_identifier = *archetype_entry.key();
            let index = unsafe {
                archetype_entry
                    .or_insert(Archetype::<R>::new(identifier_buffer))
                    .push_from_buffer_skipping_component::<C>(
                        entity_identifier,
                        current_component_bytes,
                    )
            };

            // Update the location.
            unsafe {
                self.world.entity_allocator.modify_location_unchecked(
                    entity_identifier,
                    Location::new(archetype_identifier, index),
                );
            }
        }
    }

    // pub fn query<'a, V, F>(&'a mut self) -> iter::Flatten<vec::IntoIter<V::Results>>
    // where
    //     V: Views<'a>,
    //     F: Filter,
    // {
    //     todo!()
    // }
}
