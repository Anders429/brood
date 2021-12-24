use crate::{
    component::Component, internal::entity_allocator::Location, registry::Registry, world::World,
};

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
        todo!()

        // Determine if the component already exists.
        // If so, simply replace the value.

        // If not, then do the following:
        // Pop the values from the current archetype table.
        // Append the new component to the values.
        // Insert with the new component.
        // (Is there a way to make this more efficient?)
    }

    pub fn remove<C>(&mut self)
    where
        C: Component,
    {
        todo!()

        // Determine if the component exists.
        // If not, do nothing.
        // If so, pop the values.
        // Remove the component requested.
        // Insert the rest of the components.
    }

    // TODO: Add query method.
}
