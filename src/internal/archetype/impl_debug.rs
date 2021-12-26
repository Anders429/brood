use crate::{
    entity::EntityIdentifier,
    internal::{archetype, archetype::Archetype, registry::RegistryDebug},
};
use alloc::vec::Vec;
use core::{
    fmt::{self, Debug},
    mem::ManuallyDrop,
};

struct Components<R>
where
    R: RegistryDebug,
{
    pointers: Vec<*const u8>,
    identifier: archetype::Identifier<R>,
}

impl<R> Debug for Components<R>
where
    R: RegistryDebug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_map = f.debug_map();
        unsafe {
            R::debug_components(
                &self.pointers,
                &mut debug_map,
                self.identifier.iter(),
            );
        }
        debug_map.finish()
    }
}

struct Row<R>
where
    R: RegistryDebug,
{
    identifier: EntityIdentifier,
    components: Components<R>,
}

impl<R> Debug for Row<R>
where
    R: RegistryDebug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Row")
            .field("identifier", &self.identifier)
            .field("components", &self.components)
            .finish()
    }
}

impl<R> Debug for Archetype<R>
where
    R: RegistryDebug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_map = f.debug_map();

        let entity_identifiers = ManuallyDrop::new(unsafe {
            Vec::from_raw_parts(
                self.entity_identifiers.0,
                self.length,
                self.entity_identifiers.1,
            )
        });

        for i in 0..self.length {
            let mut component_pointers = Vec::new();
            unsafe {
                R::extract_component_pointers(
                    i,
                    &self.components,
                    &mut component_pointers,
                    self.identifier_buffer.iter()
                );
            }
            debug_map.entry(
                &i,
                &Row::<R> {
                    identifier: *unsafe { entity_identifiers.get_unchecked(i) },
                    components: Components {
                        pointers: component_pointers,
                        identifier: unsafe {self.identifier_buffer.as_identifier()},
                    },
                },
            );
        }

        debug_map.finish()
    }
}
