use crate::internal::{
    archetype::Archetype,
    entity::{EntityEq, EntityPartialEq},
};

impl<E> PartialEq for Archetype<E>
where
    E: EntityPartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length
            && unsafe { E::eq(&self.components, &other.components, self.length) }
    }
}

impl<E> Eq for Archetype<E> where E: EntityEq {}
