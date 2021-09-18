use crate::{entity::Entity, internal::archetype::Archetype};

impl<E> Drop for Archetype<E>
where
    E: Entity,
{
    fn drop(&mut self) {
        unsafe {
            E::free_components(&self.components, self.length);
        }
    }
}