use crate::{archetype::Archetype, registry::Registry};
use alloc::vec::Vec;

impl<R> Drop for Archetype<R>
where
    R: Registry,
{
    #[inline]
    fn drop(&mut self) {
        unsafe {
            R::free_components(&self.components, self.length, self.identifier_buffer.iter());
        }
        unsafe {
            let _ = Vec::from_raw_parts(
                self.entity_identifiers.0,
                self.length,
                self.entity_identifiers.1,
            );
        }
    }
}
