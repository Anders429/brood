use super::World;
use crate::{
    entity::NullEntity,
    internal::registry::{RegistryEq, RegistryPartialEq},
};
use core::marker::PhantomData;

impl<R> PartialEq for World<R>
where
    R: RegistryPartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.archetypes.len() == other.archetypes.len()
            && self
                .archetypes
                .keys()
                .all(|key| other.archetypes.contains_key(key))
        {
            for key in self.archetypes.keys() {
                if !unsafe {
                    R::eq::<NullEntity, R>(
                        key,
                        0,
                        0,
                        &self.archetypes[key],
                        &other.archetypes[key],
                        PhantomData,
                        PhantomData,
                    )
                } {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

impl<R> Eq for World<R>
where
    R: RegistryEq,
{
}

#[cfg(test)]
mod tests {
    // TODO
}
